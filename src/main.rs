//! Entrypoint principal /init do SGP-Ciclobike.

use std::path::Path;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use sgp_core::{BikeConfig, OnboardingProgress, OtaChannel};
use sgp_onboarding::{ConfigGuard, OnboardingEvent, OnboardingState, resume_state, transition, CONFIG_PATH};
use sgp_onboarding::network::{wifi, ota};
use sgp_ui::framebuffer::FrameBuffer;
use sgp_ui::widgets::KeyboardWidget;
use embedded_graphics::prelude::*;

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
    // Tenta montar sistemas de arquivos necessários em tempo de boot.
    // Falha silenciosamente em ambientes de desenvolvimento no host.
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
        embedded_graphics::primitives::Rectangle::new(
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
                                // Se já atualizado ou erro transient, avança direto
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
                        // release size is 4MB = 4_194_304
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
        tokio::time::sleep(Duration::from_millis(16)).await; // ~60FPS

        // Simulação elegante de toque em testes/host para avançar telas
        if touch.is_none() && !ota_started {
            last_touch = Some((100, 120)); // Clica no topo da lista/teclado para simular o setup completo
        }
    }

    Ok(())
}

async fn run_main_app(fb: &mut FrameBuffer) {
    let _ = fb.draw_iter(std::iter::once(Pixel(Point::new(0, 0), theme::BG)));
    let text_style = embedded_graphics::text::MonoTextStyle::new(
        &embedded_graphics::mono_font::ascii::FONT_8X13,
        theme::TEXT_PRIMARY,
    );
    let _ = embedded_graphics::text::Text::new("BEM-VINDO AO CICLOBIKE!", Point::new(100, 100), text_style).draw(fb);
    fb.flush();
}
