use anyhow::{
  Result,
  Context,
};

use clap::{arg, Command};

use hala_imgui::{
  HalaApplication,
  HalaImGui,
};

use hala_renderer::{
  scene,
  renderer::HalaRendererTrait,
  drz_renderer::HalaRenderer,
};

mod config;

// The application settings.
struct MySettings {
  pub use_subpasses: bool,
  pub use_transient: bool,
  pub use_small_gbuffer: bool,
}

impl Default for MySettings {
  fn default() -> Self {
    Self {
      use_subpasses: true,
      use_transient: true,
      use_small_gbuffer: true,
    }
  }
}

/// The application.
struct MyApplication {
  log_file: String,
  config: config::AppConfig,
  settings: MySettings,
  renderer: Option<HalaRenderer>,
  imgui: Option<HalaImGui>,
}

impl Drop for MyApplication {
  fn drop(&mut self) {
    self.imgui = None;
  }
}

/// The implementation of the SDF renderer application.
impl MyApplication {

  pub fn new() -> Result<Self> {
    // Parse the command line arguments.
    let matches = cli().get_matches();
    let log_file = match matches.get_one::<String>("log") {
      Some(log_file) => log_file,
      None => "./logs/renderer.log"
    };
    let config_file = matches.get_one::<String>("config").with_context(|| "Failed to get the config file path.")?;

    // Load the configure.
    let config = config::load_app_config(config_file)?;
    log::debug!("Config: {:?}", config);
    config::validate_app_config(&config)?;

    // Create out directory.
    std::fs::create_dir_all("./out")
      .with_context(|| "Failed to create the output directory: ./out")?;

    Ok(Self {
      log_file: log_file.to_string(),
      config,
      settings: MySettings::default(),
      renderer: None,
      imgui: None,
    })
  }

}


/// The implementation of the application trait for the SDF renderer application.
impl HalaApplication for MyApplication {

  fn get_log_console_fmt(&self) -> &str {
    "{d(%H:%M:%S)} {h({l:<5})} {t:<20.20} - {m}{n}"
  }
  fn get_log_file_fmt(&self) -> &str {
    "{d(%Y-%m-%d %H:%M:%S)} {h({l:<5})} {f}:{L} - {m}{n}"
  }
  fn get_log_file(&self) -> &std::path::Path {
    std::path::Path::new(self.log_file.as_str())
  }
  fn get_log_file_size(&self) -> u64 {
    1024 * 1024 /* 1MB */
  }
  fn get_log_file_roller_count(&self) -> u32 {
    5
  }

  fn get_window_title(&self) -> &str {
    "Derfered Renderer"
  }
  fn get_window_size(&self) -> winit::dpi::PhysicalSize<u32> {
    winit::dpi::PhysicalSize::new(self.config.window.width as u32, self.config.window.height as u32)
  }

  fn get_imgui(&self) -> Option<&HalaImGui> {
    self.imgui.as_ref()
  }
  fn get_imgui_mut(&mut self) -> Option<&mut HalaImGui> {
    self.imgui.as_mut()
  }

  /// The before run function.
  /// param width: The width of the window.
  /// param height: The height of the window.
  /// param window: The window.
  /// return: The result.
  fn before_run(&mut self, _width: u32, _height: u32, window: &winit::window::Window) -> Result<()> {
    let now = std::time::Instant::now();
    let mut scene = scene::cpu::HalaScene::new(&self.config.scene_file)?;
    log::info!("Load scene used {}ms.", now.elapsed().as_millis());

    // Setup the renderer.
    let gpu_req = hala_gfx::HalaGPURequirements {
      width: self.config.window.width as u32,
      height: self.config.window.height as u32,
      version: (1, 3, 0),
      require_srgb_surface: true,
      require_mesh_shader: true,
      require_ray_tracing: false,
      require_10bits_output: false,
      is_low_latency: true,
      require_depth: true,
      require_printf_in_shader: cfg!(debug_assertions),
      ..Default::default()
    };

    // Create the renderer.
    let mut renderer = HalaRenderer::new(
      "Deferred Renderer",
      &gpu_req,
      window,
    )?;
    renderer.create_gbuffer_images(
      self.settings.use_transient,
      if self.settings.use_small_gbuffer {
        hala_gfx::HalaFormat::R8G8B8A8_UNORM
      } else {
        hala_gfx::HalaFormat::R32G32B32A32_SFLOAT
      },
      if self.settings.use_small_gbuffer {
        hala_gfx::HalaFormat::A2R10G10B10_UNORM_PACK32
      } else {
        hala_gfx::HalaFormat::R32G32B32A32_SFLOAT
      },
    )?;

    let shaders_dir = if cfg!(debug_assertions) {
      "shaders/output/debug/hala-subpasses/HALA_SUBPASSES"
    } else {
      "shaders/output/release/hala-subpasses/HALA_SUBPASSES"
    };
    renderer.push_shaders_with_file(
      Some(&format!("{}/default.as_6_8.spv", shaders_dir)),
      &format!("{}/default.ms_6_8.spv", shaders_dir),
      &format!("{}/default.ps_6_8.spv", shaders_dir),
      "default",
    )?;

    renderer.set_scene(&mut scene)?;

    renderer.commit()?;

    // Setup the imgui.
    self.imgui = Some(HalaImGui::new(
      std::rc::Rc::clone(&(*renderer.resources().context)),
      false,
    )?);

    self.renderer = Some(renderer);

    Ok(())
  }

  /// The after run function.
  fn after_run(&mut self) {
    if let Some(renderer) = &mut self.renderer.take() {
      renderer.wait_idle().expect("Failed to wait the renderer idle.");
      self.imgui = None;
    }
  }

  /// The update function.
  /// param delta_time: The delta time.
  /// return: The result.
  fn update(&mut self, delta_time: f64, width: u32, height: u32) -> Result<()> {
    if let Some(imgui) = self.imgui.as_mut() {
      imgui.begin_frame(
        delta_time,
        width,
        height,
        |ui| {
          if let Some(_renderer) = self.renderer.as_mut() {
            ui.window("Derfered Renderer")
              .collapsed(false, imgui::Condition::FirstUseEver)
              .position([10.0, 10.0], imgui::Condition::FirstUseEver)
              .always_auto_resize(true)
              .build(|| {
                let _ = ui.checkbox("Use Subpasses", &mut self.settings.use_subpasses);
                let _ = ui.checkbox("Use Transient", &mut self.settings.use_transient);
                let _ = ui.checkbox("Use Small(128-bits) G-Buffer", &mut self.settings.use_small_gbuffer);
              }
            );
          }
        }
      )?;
      imgui.end_frame()?;
    }

    if let Some(renderer) = &mut self.renderer {
      renderer.update(
        delta_time,
        width,
        height,
        |index, command_buffers| {
          if let Some(imgui) = self.imgui.as_mut() {
            imgui.draw(index, command_buffers)?;
          }

          Ok(())
        }
      )?;
    }

    Ok(())
  }

  /// The render function.
  /// return: The result.
  fn render(&mut self) -> Result<()> {
    if let Some(renderer) = &mut self.renderer {
      renderer.render()?;
    }

    Ok(())
  }

}


/// The command line interface.
fn cli() -> Command {
  Command::new("hala-subpasses")
    .about("The Deferred Renderer based on Vulkan subpasses.")
    .arg_required_else_help(true)
    .arg(arg!(-l --log <LOG_FILE> "The file path of the log file. Default is ./logs/renderer.log."))
    .arg(arg!(-c --config [CONFIG_FILE] "The file path of the config file."))
}

/// The normal main function.
fn main() -> Result<()> {
  // Initialize the application.
  let mut app = MyApplication::new()?;
  app.init()?;

  // Run the application.
  app.run()?;

  Ok(())
}
