#[cfg(target_os = "android")]
mod android {
    use zng::{view_process::default::*, prelude::*};

    // Android entry point.
    #[no_mangle]
    fn android_main(app: android::AndroidApp) {
        zng::env::init!();
        zng::app::print_tracing(tracing::Level::INFO);
        android::init_android_app(app.clone());
        run_same_process(|| {
            APP.defaults()
                .run_window(async { gui::primary::window().await });
        });
    }
}
