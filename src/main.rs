use std::path::Path;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, watch};

use sgp_core::{BikeConfig, OnboardingProgress, OtaChannel};
use sgp_onboarding::{ConfigGuard, OnboardingEvent, OnboardingState, resume_state, transition, CONFIG_PATH};
use sgp_onboarding::network::{wifi, ota};
use sgp_ui::framebuffer::FrameBuffer;
use sgp_ui::widgets::{KeyboardWidget, EmergencyAlertWidget};
use sgp_sensors::{SensorData, SensorReader, MockSensor, mock::MockScenario};
use sgp_safety::{FallDetector, ImuSample, AlertManager, AlertState, AlertInput, EmergencyDispatcher};
use sgp_telemetry::{SessionManager, SyncWorker};

use embedded_graphics::prelude::*;
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::primitives::{Rectangle, PrimitiveStyleBuilder};
use embedded_graphics::text::Text;
use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};

use sgp_ui::widgets::{
    KeyboardWidget, EmergencyAlertWidget, SpeedometerWidget,
    MetricPanelWidget, StatusBarWidget, GpsPanelWidget, ActionButtonWidget,
};
use sgp_telemetry::SessionSummary;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    tracing::info!("Iniciando SGP-Ciclobike /init...");

    mount_essential_filesystems();
    
    let mut fb = FrameBuffer::open();
    let config_path = Path::new(CONFIG_PATH);
    let mut config_guard = ConfigGuard::load_or_default(config_path);

    if config_guard.progress().setup_complete {
        tracing::info!("Onboarding completo. Iniciando aplicação principal.");
        run_main_app(&mut fb).await;
        return Ok(());
    }

    run_onboarding_wizard(&mut fb, &mut config_guard).await?;
    Ok(())
}

fn mount_essential_filesystems() {
    let _ = std::fs::create_dir_all("/proc");
    let _ = std::fs::create_dir_all("/sys");
    let _ = std::fs::create_dir_all("/dev");
}

async fn run_onboarding_wizard(
    fb: &mut FrameBuffer,
    config_guard: &mut ConfigGuard,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut state = resume_state(config_guard.progress());
    let mut keyboard = KeyboardWidget::new(
        Rectangle::new(
            Point::new(0, 500),
            Size::new(540, 360),
        ),
    );

    let mut last_touch = None;
    let mut ota_started = false;
    let mut ota_progress = 0u32;
    let mut ota_status = "Verificando...".to_string();
    let (ota_tx, mut ota_rx) = mpsc::channel::<u64>(10);

    loop {
        let touch = last_touch.take();

        match &state {
            OnboardingState::SelectLanguage => {
                if let Some(lang) = sgp_onboarding::screen::s01_language::run(fb, touch) {
                    let _ = config_guard.save_step(|p| p.language = Some(lang.clone()));
                    state = transition(state, OnboardingEvent::LanguageSelected(lang)).unwrap_or_else(|(s, _)| s);
                }
            }
            OnboardingState::SelectCountry { .. } => {
                if let Some(country) = sgp_onboarding::screen::s02_country::run(fb, touch) {
                    let _ = config_guard.save_step(|p| p.country = Some(country.clone()));
                    state = transition(state, OnboardingEvent::CountrySelected(country)).unwrap_or_else(|(s, _)| s);
                }
            }
            OnboardingState::ConnectWifi { .. } => {
                if let Some(ssid) = sgp_onboarding::screen::s03_wifi::run(fb, &mut keyboard, touch) {
                    let _ = wifi::connect_wifi(&ssid, keyboard.text()).await;
                    let _ = config_guard.save_step(|p| p.wifi_ssid = Some(ssid.clone()));
                    state = transition(state, OnboardingEvent::WifiConnected { ssid }).unwrap_or_else(|(s, _)| s);
                }
            }
            OnboardingState::CheckOtaUpdate { wifi_ssid, .. } => {
                sgp_onboarding::screen::s04_ota::run(fb, ota_progress, &ota_status);

                if !ota_started {
                    ota_started = true;
                    let current_ver = semver::Version::parse("0.1.0").unwrap();
                    let tx = ota_tx.clone();
                    let ssid = wifi_ssid.clone();

                    tokio::spawn(async move {
                        tracing::info!("Iniciando busca OTA na rede: {}", ssid);
                        let release = ota::check_ota_update(&current_ver, OtaChannel::Release, "https://api.ciclobike.com").await;
                        match release {
                            Ok(Some(r)) => {
                                let _ = ota::download_and_apply(&r, tx).await;
                            }
                            _ => {
                                let _ = tx.send(u64::MAX).await;
                            }
                        }
                    });
                }

                while let Ok(bytes) = ota_rx.try_recv() {
                    if bytes == u64::MAX {
                        ota_progress = 100;
                        ota_status = "Sistema atualizado!".to_string();
                        tokio::time::sleep(Duration::from_millis(500)).await;
                        
                        let _ = config_guard.save_step(|p| {
                            p.ota_checked = Some(true);
                            let _ = p.try_finalize();
                        });

                        state = transition(state, OnboardingEvent::OtaCheckDone { update_applied: false }).unwrap_or_else(|(s, _)| s);
                    } else {
                        ota_progress = ((bytes * 100) / 4_194_304) as u32;
                        ota_status = "Baixando atualizacao...".to_string();
                    }
                }
            }
            OnboardingState::Complete => {
                tracing::info!("Setup finalizado!");
                break;
            }
        }

        fb.flush();
        tokio::time::sleep(Duration::from_millis(16)).await;

        if touch.is_none() && !ota_started {
            last_touch = Some((100, 120));
        }
    }

    Ok(())
}

async fn run_main_app(fb: &mut FrameBuffer) {
    tracing::info!("Inicializando ecossistema de sensores, segurança e telemetria...");

    let mut mock_sensor = MockSensor::new(MockScenario::NormalBiking);

    let (alert_input_tx, alert_input_rx) = mpsc::channel::<AlertInput>(10);
    let (alert_state_tx, mut alert_state_rx) = watch::channel::<AlertState>(AlertState::Idle);
    let (emergency_trigger_tx, emergency_trigger_rx) = mpsc::channel::<()>(10);
    let (fall_detected_tx, fall_detected_rx) = mpsc::channel::<()>(10);

    let dispatcher = EmergencyDispatcher::new(None, "192", Some("https://api.ciclobike.com/emergency"));
    tokio::spawn(dispatcher.run(emergency_trigger_rx));

    let alert_manager = AlertManager::new(alert_input_rx, alert_state_tx, emergency_trigger_tx);
    tokio::spawn(alert_manager.run(fall_detected_rx));

    let telemetry_path = Path::new("/tmp/sgp_telemetry.json");
    let mut session_manager = SessionManager::new(telemetry_path);
    session_manager.start_session();

    let sync_worker = SyncWorker::new(telemetry_path, None, 1883, Some("https://api.ciclobike.com/telemetry"));
    tokio::spawn(sync_worker.run());

    let screen_bounds = Rectangle::new(Point::new(0, 0), Size::new(540, 960));
    let alert_widget = EmergencyAlertWidget::new(screen_bounds, "192");

    let mut speedometer = SpeedometerWidget::new(Rectangle::new(Point::new(20, 40), Size::new(500, 280)));
    let mut cadence_panel = MetricPanelWidget::new(Rectangle::new(Point::new(20, 330), Size::new(240, 100)), "CADÊNCIA", "RPM");
    let mut distance_panel = MetricPanelWidget::new(Rectangle::new(Point::new(280, 330), Size::new(240, 100)), "DISTÂNCIA", "km");
    let mut altitude_panel = MetricPanelWidget::new(Rectangle::new(Point::new(20, 440), Size::new(240, 100)), "ALTITUDE", "m");
    let mut gps_panel = GpsPanelWidget::new(Rectangle::new(Point::new(280, 440), Size::new(240, 100)));
    let mut status_bar = StatusBarWidget::new(540);

    let mut last_flash = Instant::now();
    let mut flash_state = false;
    let mut simulate_fall_active = false;
    let mut last_touch = None;

    let mut speed_val = 20.0f32;
    let mut cadence_val = 80.0f32;
    let mut sat_count = 8u8;
    let mut distance_val = 0.0f32;
    let mut altitude_val = 760.0f32;
    let mut lat_val = -23.5505f64;
    let mut lon_val = -46.6333f64;
    let mut last_tick = Instant::now();
    let mut session_seconds = 0u32;
    let mut is_recording = true;
    let mut active_summary: Option<SessionSummary> = None;

    let mut fall_detector = FallDetector::default();

    loop {
        let touch = last_touch.take();
        let now = Instant::now();

        if now.duration_since(last_flash) >= Duration::from_millis(500) {
            flash_state = !flash_state;
            last_flash = now;
        }

        if let Ok(data) = mock_sensor.read().await {
            match data {
                SensorData::Imu { accel_x, accel_y, accel_z, .. } => {
                    if let Some(_fall_event) = fall_detector.feed(ImuSample {
                        timestamp: now,
                        accel_x,
                        accel_y,
                        accel_z,
                    }) {
                        let _ = fall_detected_tx.send(()).await;
                    }
                }
                SensorData::Speed { rpm, speed_kmh } => {
                    if is_recording {
                        speed_val = speed_kmh;
                        cadence_val = rpm / 2.0;
                    }
                }
                SensorData::Gps { lat, lon, altitude_m, speed_kmh: _, satellites } => {
                    if is_recording {
                        sat_count = satellites;
                        altitude_val = altitude_m;
                        lat_val = lat;
                        lon_val = lon;
                    }
                }
            }
        }

        if is_recording && now.duration_since(last_tick) >= Duration::from_secs(1) {
            session_seconds += 1;
            let elapsed_hours = 1.0 / 3600.0;
            distance_val += speed_val * elapsed_hours;
            
            let _ = session_manager.record_point(
                Some(lat_val),
                Some(lon_val),
                Some(altitude_val),
                speed_val,
                cadence_val,
                9.8,
            );
            last_tick = now;
        }

        speedometer.set_speed(if is_recording { speed_val } else { 0.0 });
        cadence_panel.set_value(if is_recording { cadence_val } else { 0.0 });
        distance_panel.set_value(distance_val);
        altitude_panel.set_value(if is_recording { altitude_val } else { 0.0 });
        gps_panel.update(lat_val, lon_val, altitude_val, sat_count);
        status_bar.update(sat_count, session_seconds, is_recording);

        let current_state = *alert_state_rx.borrow();

        match current_state {
            AlertState::Alerting { seconds_remaining } => {
                let _ = alert_widget.draw(fb, seconds_remaining, flash_state);

                if let Some((tx, ty)) = touch {
                    if alert_widget.check_touch(Point::new(tx, ty)) {
                        let _ = alert_input_tx.send(AlertInput::Cancel).await;
                        simulate_fall_active = false;
                        mock_sensor.set_scenario(MockScenario::NormalBiking);
                    }
                }
            }
            AlertState::Idle | AlertState::EmergencyTriggered => {
                render_dashboard(
                    fb,
                    &speedometer,
                    &cadence_panel,
                    &distance_panel,
                    &altitude_panel,
                    &gps_panel,
                    &status_bar,
                    is_recording,
                    &active_summary,
                );

                if let Some((tx, ty)) = touch {
                    let p = Point::new(tx, ty);

                    let bounds_pause = Rectangle::new(Point::new(20, 550), Size::new(240, 70));
                    if bounds_pause.contains(p) {
                        if is_recording {
                            is_recording = false;
                            speed_val = 0.0;
                            cadence_val = 0.0;
                            if let Ok(summary) = session_manager.stop_session() {
                                active_summary = Some(summary);
                            }
                        } else {
                            is_recording = true;
                            active_summary = None;
                            session_manager.start_session();
                            last_tick = now;
                        }
                    }

                    let bounds_fall = Rectangle::new(Point::new(280, 550), Size::new(240, 70));
                    if bounds_fall.contains(p) {
                        tracing::warn!("Iniciando simulação de queda no sensor mock...");
                        simulate_fall_active = true;
                        mock_sensor.set_scenario(MockScenario::FallTrigger);
                        let _ = fall_detected_tx.send(()).await;
                    }
                }
            }
        }

        fb.flush();
        tokio::time::sleep(Duration::from_millis(16)).await;

        if !simulate_fall_active && now.duration_since(last_flash) > Duration::from_secs(60) {
            last_touch = Some((300, 580));
        }
    }
}

fn render_dashboard(
    fb: &mut FrameBuffer,
    speedometer: &SpeedometerWidget,
    cadence_panel: &MetricPanelWidget,
    distance_panel: &MetricPanelWidget,
    altitude_panel: &MetricPanelWidget,
    gps_panel: &GpsPanelWidget,
    status_bar: &StatusBarWidget,
    is_recording: bool,
    active_summary: &Option<SessionSummary>,
) {
    let _ = fb.draw_iter(std::iter::once(Pixel(Point::new(0, 0), sgp_ui::theme::BG)));

    let screen_bounds = Rectangle::new(Point::new(0, 0), Size::new(540, 960));
    let fill_style = PrimitiveStyleBuilder::new()
        .fill_color(sgp_ui::theme::BG)
        .build();
    let _ = screen_bounds.draw_styled(&fill_style, fb);

    let _ = status_bar.draw(fb);
    let _ = speedometer.draw(fb);
    let _ = cadence_panel.draw(fb);
    let _ = distance_panel.draw(fb);
    let _ = altitude_panel.draw(fb);
    let _ = gps_panel.draw(fb);

    let pause_label = if is_recording { "PAUSAR" } else { "RETOMAR" };
    let pause_button = ActionButtonWidget::new(
        Rectangle::new(Point::new(20, 550), Size::new(240, 70)),
        pause_label,
        sgp_ui::theme::ACCENT_ALT,
    );
    let _ = pause_button.draw(fb);

    let fall_button = ActionButtonWidget::new(
        Rectangle::new(Point::new(280, 550), Size::new(240, 70)),
        "SIM. QUEDA",
        sgp_ui::theme::ERROR,
    );
    let _ = fall_button.draw(fb);

    if let Some(summary) = active_summary {
        render_session_summary(fb, summary);
    }
}

fn render_session_summary(fb: &mut FrameBuffer, summary: &SessionSummary) {
    let summary_bounds = Rectangle::new(Point::new(20, 640), Size::new(500, 280));
    let card_style = PrimitiveStyleBuilder::new()
        .fill_color(sgp_ui::theme::BG_CARD)
        .stroke_color(sgp_ui::theme::ACCENT)
        .stroke_width(2)
        .build();
    let _ = summary_bounds.draw_styled(&card_style, fb);

    let label_style = MonoTextStyle::new(&FONT_10X20, sgp_ui::theme::TEXT_PRIMARY);
    let data_style = MonoTextStyle::new(&FONT_10X20, sgp_ui::theme::TEXT_SECONDARY);

    let _ = Text::new("RESUMO DA SESSÃO", Point::new(40, 675), label_style).draw(fb);
    
    let line1 = format!("Distancia: {:.2} km", summary.total_distance_km);
    let _ = Text::new(&line1, Point::new(40, 715), data_style).draw(fb);

    let h = summary.duration_seconds / 3600;
    let m = (summary.duration_seconds % 3600) / 60;
    let s = summary.duration_seconds % 60;
    let line2 = format!("Tempo: {:02}:{:02}:{:02}", h, m, s);
    let _ = Text::new(&line2, Point::new(40, 755), data_style).draw(fb);

    let line3 = format!("Vel. Media: {:.1} km/h", summary.average_speed_kmh);
    let _ = Text::new(&line3, Point::new(40, 795), data_style).draw(fb);

    let line4 = format!("Vel. Maxima: {:.1} km/h", summary.max_speed_kmh);
    let _ = Text::new(&line4, Point::new(40, 835), data_style).draw(fb);
}
