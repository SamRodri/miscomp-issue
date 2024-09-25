#[cfg(target_os = "android")]
mod android {
    use zng::{
        event::{event, event_args},
        prelude::*,
        view_process::default::*,
    };

    // Android entry point.
    #[no_mangle]
    fn android_main(app: android::AndroidApp) {
        zng::env::init!();
        zng::app::print_tracing(tracing::Level::INFO);
        android::init_android_app(app.clone());
        run_same_process(|| {
            APP.defaults().run(async {
                test("app-start");

                TEST_EVENT
                    .on_pre_event(app_hn!(|_: &TestArgs, _| {
                        // this fails (prints None)
                        test("test-event");
                    }))
                    .perm();
                TEST_EVENT.notify(TestArgs::now());

                task::deadline(1.secs()).await;
                APP.exit()
            });
        });
    }

    pub fn test(ctx: &str) {
        use zng::layout::*;
        // (Rect(3240pxx7200px at (-1080px, -2400px)), Rect(1080pxx90px at (0px, 0px))
        let cull = std::hint::black_box(PxRect::new(
            PxPoint::new(Px(-1080), Px(-2400)),
            PxSize::new(Px(3240), Px(7200)),
        ));
        let bounds = std::hint::black_box(PxRect::new(
            PxPoint::new(Px(0), Px(0)),
            PxSize::new(Px(1080), Px(90)),
        ));
        let i = cull.intersection(&bounds);
        println!("!!: {ctx} {:?}", i);
    }

    event_args! {
        pub struct TestArgs {

            ..

            fn delivery_list(&self, list: &mut UpdateDeliveryList) {
                list.search_all();
            }
        }
    }
    event! {
        static TEST_EVENT: TestArgs;
    }
}
