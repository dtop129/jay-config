use chrono::{format::StrftimeItems, Local};
use jay_config::{
    config, exec::Command, get_workspace, input::{get_default_seat, input_devices, on_new_input_device, FocusFollowsMouseMode, InputDevice}, keyboard::{mods::{Modifiers, ALT, CTRL, MOD4, SHIFT}, parse_keymap, syms::*}, quit, reload, status::set_status, switch_to_vt, timer::{duration_until_wall_clock_is_multiple_of, get_timer}, video::{on_graphics_initialized, set_gfx_api}
};
use std::{cell::RefCell, rc::Rc, time::Duration};

fn setup_status() {
    let time_format: Vec<_> = StrftimeItems::new("%Y-%m-%d %H:%M").collect();

    let update_status = move || {
        let status = format!("{}", Local::now().format_with_items(time_format.iter()));

        set_status(&status);
    };

    update_status();

    let period = Duration::from_secs(5);
    let timer = get_timer("status_timer");
    timer.repeated(duration_until_wall_clock_is_multiple_of(period), period);
    timer.on_tick(update_status);
}

fn configure() {
    let hostname = hostname::get().unwrap();
    let seat = get_default_seat();

    if hostname == "dtopPC2" {
        set_gfx_api(jay_config::video::GfxApi::OpenGl);
        seat.set_keymap(parse_keymap(include_str!("keymap_us.xkb")));
    } else {
        set_gfx_api(jay_config::video::GfxApi::Vulkan);
        seat.set_keymap(parse_keymap(include_str!("keymap_it.xkb")));
    }

    on_new_input_device(|device: InputDevice| {
        device.set_natural_scrolling_enabled(true);
        device.set_tap_enabled(true);
    });

    seat.set_focus_follows_mouse_mode(FocusFollowsMouseMode::True);
    seat.set_repeat_rate(60, 250);
    seat.bind(MOD4 | SHIFT | SYM_q, || quit());
    seat.bind(MOD4 | SHIFT | SYM_r, || {
        Command::new("notify")
            .arg("Jay")
            .arg("Reloading config")
            .spawn();
        reload()
    });
    seat.bind(MOD4 | SYM_q, move || {
        seat.close();
    });
    seat.bind(MOD4 | SYM_h, move || {
        seat.focus(jay_config::Direction::Left)
    });
    seat.bind(MOD4 | SYM_l, move || {
        seat.focus(jay_config::Direction::Right)
    });
    seat.bind(MOD4 | SYM_j, move || {
        seat.focus(jay_config::Direction::Down)
    });
    seat.bind(MOD4 | SYM_k, move || seat.focus(jay_config::Direction::Up));
    seat.bind(MOD4 | SHIFT | SYM_h, move || {
        seat.move_(jay_config::Direction::Left)
    });
    seat.bind(MOD4 | SHIFT | SYM_l, move || {
        seat.move_(jay_config::Direction::Right)
    });
    seat.bind(MOD4 | SHIFT | SYM_j, move || {
        seat.move_(jay_config::Direction::Down)
    });
    seat.bind(MOD4 | SHIFT | SYM_k, move || {
        seat.move_(jay_config::Direction::Up)
    });
    seat.bind(MOD4 | SYM_f, move || seat.toggle_fullscreen());
    seat.bind(MOD4 | SYM_s, move || {
        seat.create_split(jay_config::Axis::Horizontal);
    });
    seat.bind(MOD4 | SYM_v, move || {
        seat.create_split(jay_config::Axis::Vertical);
    });
    let fn_keys = [
        SYM_F1, SYM_F2, SYM_F3, SYM_F4, SYM_F5, SYM_F6, SYM_F7, SYM_F8, SYM_F9,
    ];
    for (i, sym) in fn_keys.into_iter().enumerate() {
        seat.bind(CTRL | ALT | sym, move || switch_to_vt(i as u32 + 1));
    }

    let num_keys = [
        SYM_1, SYM_2, SYM_3, SYM_4, SYM_5, SYM_6, SYM_7, SYM_8, SYM_9,
    ];
    let ws_hist = Rc::new(RefCell::new((0, 0)));
    for (i, sym) in num_keys.into_iter().enumerate() {
        let ws = get_workspace(&format!("{}", i + 1));
        seat.bind(MOD4 | sym, {
            let ws_hist = ws_hist.clone();
            move || {
                let wss = &mut ws_hist.borrow_mut();
                if wss.1 == i {
                    return;
                }

                wss.0 = wss.1;
                wss.1 = i;
                seat.show_workspace(ws);
            }
        });

        seat.bind(MOD4 | SHIFT | sym, move || {
            seat.set_workspace(ws);
        });
    }

    seat.bind(MOD4 | SYM_Tab, move || {
        let wss = &mut ws_hist.borrow_mut();
        let ws = get_workspace(&format!("{}", wss.0 + 1));

        let tmp = wss.0;
        wss.0 = wss.1;
        wss.1 = tmp;
        seat.show_workspace(ws);
    });

    seat.bind_masked(Modifiers::NONE, SYM_XF86AudioLowerVolume, || {
        Command::new("wpctl")
            .arg("set-volume")
            .arg("-l")
            .arg("1.5")
            .arg("@DEFAULT_AUDIO_SINK@")
            .arg("2%-")
            .spawn();
    });
    seat.bind_masked(Modifiers::NONE, SYM_XF86AudioRaiseVolume, || {
        Command::new("wpctl")
            .arg("set-volume")
            .arg("-l")
            .arg("1.5")
            .arg("@DEFAULT_AUDIO_SINK@")
            .arg("2%+")
            .spawn();
    });
    seat.bind_masked(Modifiers::NONE, SYM_XF86AudioMute, || {
        Command::new("wpctl")
            .arg("set-mute")
            .arg("@DEFAULT_AUDIO_SINK@")
            .arg("toggle")
            .spawn();
    });
    seat.bind(MOD4 | SYM_p, || Command::new("fuzzel").spawn());
    seat.bind(MOD4 | SHIFT | SYM_Return, || {
        Command::new("footclient").spawn()
    });
    seat.bind(MOD4 | CTRL | SYM_a, || {
        Command::new("dmenuplaylist").spawn()
    });
    seat.bind(MOD4 | CTRL | SYM_b, || Command::new("firefox").spawn());
    seat.bind(MOD4 | CTRL | SYM_f, || {
        Command::new("footclient").arg("lf").spawn()
    });
    seat.bind(MOD4 | CTRL | SYM_m, || {
        Command::new("start_torrserver")
            .arg("-n")
            .arg("9090")
            .arg(".local/state/mpv/torrserver")
            .spawn();
    });
    seat.bind(MOD4 | CTRL | SYM_n, || Command::new("dmenuumount").spawn());
    seat.bind(MOD4 | CTRL | SYM_r, || {
        Command::new("mangareader.py").spawn()
    });
    seat.bind(MOD4 | CTRL | SYM_t, || Command::new("dmenutorrent").spawn());
    seat.bind(MOD4 | CTRL | SYM_x, || Command::new("dmenupower").spawn());

    on_graphics_initialized(|| {
        Command::new("foot").arg("--server").spawn();
    });

    setup_status();
}

config!(configure);
