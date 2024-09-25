use zng::{
    event::{event, event_args},
    prelude::*,
};

pub async fn window() -> window::WindowRoot {
    // this works (prints Some(_))
    test("app-start");

    TEST_EVENT
        .on_pre_event(app_hn!(|_: &TestArgs, _| {
            // this fails (prints None)
            test("test-event");
        }))
        .perm();

    // std::thread::spawn(|| {
    // });
    TEST_EVENT.notify(TestArgs::now());

    Window! {
        id = "primary-window";
        title = zng::env::about().app.clone();
        icon = shared::res::ICON_SMALL;

        child_top = menu(), 0;
        child = content();
    }
}

// #[zng::hot_reload::hot_node]
fn menu() -> impl UiNode {
    Menu!(ui_vec![
        SubMenu!(
            l10n!("primary/menu-edit", "Edit"),
            ui_vec![Button!(zng::config::settings::SETTINGS_CMD)]
        ),
        SubMenu!(
            l10n!("primary/menu-about", "About"),
            ui_vec![
                #[cfg(feature = "dev")]
                Button!(zng::window::cmd::INSPECT_CMD.scoped(WINDOW.id())),
                Button!(zng::third_party::OPEN_LICENSES_CMD),
            ],
        ),
    ])
}

// #[zng::hot_reload::hot_node]
fn content() -> impl UiNode {
    Stack! {
        layout::align = Align::CENTER;
        direction = StackDirection::top_to_bottom();
        children = ui_vec![
            Image!(shared::res::ICON_MEDIUM),
            Text! {
                txt = l10n!("primary_win/greetings", "Hello miscomp-issue!");
                font_size = 2.em();
                txt_align = Align::CENTER;
            },
        ]
    }
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
