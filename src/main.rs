extern crate cairo;
extern crate clap;
extern crate hound;
extern crate image;

mod args;
mod feed;
mod ghostweb;
mod render;

use args::Args;
use args::ColorMode;
use args::Method;
use cairo::{Context, Format, ImageSurface};
use clap::Parser;
use degenerate::{load_soundfile, ramp, save_frame};
use ghostweb::{ghostweb, load_image};
use pbr::ProgressBar;
use std::convert::TryInto;

fn main() {
    let args = Args::parse();
    let radius = if args.radius > 0. {
        args.radius
    } else {
        (args.width / 2) as f64
    };
    multi_frame(radius, args)
}

fn multi_frame(radius: f64, args: Args) {
    let frames: usize;
    let duration: f64;
    let blocksize: usize;
    let samples: Vec<i32>;
    let mut radius = radius;
    let image = if args.image.is_empty() {
        None
    } else {
        load_image(&args.image, args.scale_image)
    };
    let (is, xs): (u32, Vec<ghostweb::Feed>) = match image {
        None => (0, vec![]),
        Some((is, xs)) => (is, xs),
    };

    let iterations = if args.iterations > 0 {
        args.iterations
    } else {
        match is {
            0 => args.width * args.height,
            _ => is,
        }
    };

    if args.soundfile.is_empty() {
        blocksize = 255;
        frames = if args.frames > 0 { args.frames } else { 1 };
        duration = frames as f64 / args.fps as f64;
        samples = ramp(blocksize * frames);
    } else {
        // the compiler doesn't like destructuring assignment
        let result = load_soundfile(args.soundfile.clone(), args.fps, args.frames, args.debug);
        blocksize = result.0;
        frames = result.1;
        duration = result.2;
        samples = result.3;
    }
    let mut block_iterator = samples.chunks(blocksize).skip(args.start);

    let basename = args.filename.clone();
    let outdir = args.outdir.clone();
    let mut pb = ProgressBar::new(frames as u64);
    let end = args.start + frames;

    for i in args.start..end {
        let block: Vec<i32>;
        let t = i as f64 / duration as f64 * args.t;
        let filename = format!("{}{}", basename, format!("{:01$}", i, 6));
        radius = radius * args.expansion;

        block = block_iterator
            .next()
            .unwrap()
            .try_into()
            .expect("could not unwrap soundfile sample block");

        let config =
            render::RenderConfig::new(iterations, args.method.clone(), radius, block, t, &args);
        let frame = match xs[..] {
            [] => render_frame(config, args.debug),
            _ => render_displacement_frame(config, &xs, i as f64 / frames as f64, args.debug),
        };
        save_frame(frame, &outdir, &filename);
        pb.inc();
    }
    pb.finish_print("done!");
}

fn render_frame(conf: render::RenderConfig, debug: bool) -> ImageSurface {
    let surface =
        ImageSurface::create(Format::ARgb32, conf.width as i32, conf.height as i32).unwrap();
    let context = Context::new(&surface).unwrap();
    let xs = ghostweb(
        conf.iterations,
        &conf.block,
        conf.radius,
        conf.f1,
        conf.f2,
        conf.m,
        conf.t,
    );
    draw_frame(
        &context,
        &xs,
        conf.width,
        conf.height,
        conf.size,
        debug,
        &conf.method,
        conf.combine_dots,
        &conf.color_mode,
        conf.saturation,
        conf.brightness,
    );
    surface
}

fn displace(
    pixels: &Vec<ghostweb::Feed>,
    dx: &Vec<ghostweb::Feed>,
    strength: f64,
) -> Vec<ghostweb::Feed> {
    pixels
        .into_iter()
        .zip(dx)
        .map(|(p, x)| ghostweb::Feed {
            p1: ghostweb::Point {
                x: p.p1.x * (1. - strength) + x.p1.x * strength,
                y: p.p1.y * (1. - strength) + x.p1.y * strength,
                z: p.p1.z * (1. - strength) + x.p1.z * strength,
            },
            p2: ghostweb::Point {
                x: p.p1.x * (1. - strength) + x.p2.x * strength,
                y: p.p1.y * (1. - strength) + x.p2.y * strength,
                z: p.p1.z * (1. - strength) + x.p2.z * strength,
            },
            radius: p.radius * (1. - strength) + x.radius * strength,
        })
        .collect()
}

fn render_displacement_frame(
    conf: render::RenderConfig,
    pixels: &Vec<ghostweb::Feed>,
    strength: f64,
    debug: bool,
) -> ImageSurface {
    let surface =
        ImageSurface::create(Format::ARgb32, conf.width as i32, conf.height as i32).unwrap();
    let context = Context::new(&surface).unwrap();
    let xs = ghostweb(
        conf.iterations,
        &conf.block,
        conf.radius,
        conf.f1,
        conf.f2,
        conf.m,
        conf.t,
    );
    draw_frame(
        &context,
        &displace(&pixels, &xs, strength),
        conf.width,
        conf.height,
        conf.size,
        debug,
        &conf.method,
        conf.combine_dots,
        &conf.color_mode,
        conf.saturation,
        conf.brightness,
    );
    surface
}

fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let c = v * s;
    let h_prime = (h % 360.0) / 60.0;
    let x = c * (1.0 - ((h_prime % 2.0) - 1.0).abs());
    let (r1, g1, b1) = match h_prime as i32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = v - c;
    (r1 + m, g1 + m, b1 + m)
}

fn draw_frame(
    context: &Context,
    xs: &Vec<ghostweb::Feed>,
    width: u32,
    height: u32,
    size: f64,
    debug: bool,
    method: &Method,
    combine_dots: bool,
    color_mode: &ColorMode,
    saturation: f64,
    brightness: f64,
) {
    let cx: f64 = width as f64 / 2.;
    let cy: f64 = height as f64 / 2.;

    // black out
    context.set_source_rgb(0.0, 0.0, 0.0);
    context.paint().unwrap();

    let total = xs.len() as f64;
    for (idx, x) in xs.iter().enumerate() {
        if debug {
            println!("{:?}", x);
        }

        let crx1 = cx + x.p1.x * x.radius;
        let cry1 = cy + x.p1.y * x.radius;
        let crx2 = cx + x.p2.x * x.radius;
        let cry2 = cy + x.p2.y * x.radius;
        let crx3 = cx + cx * x.p1.x + x.p2.x * x.radius;
        let cry3 = cy + cy * x.p1.y + x.p2.y * x.radius;

        // Calculate color based on mode
        let (r, g, b) = match color_mode {
            ColorMode::Mono => (1.0, 1.0, 1.0),
            ColorMode::Rainbow => {
                let hue = (idx as f64 / total) * 360.0;
                hsv_to_rgb(hue, saturation, brightness)
            }
            ColorMode::Frequency => {
                let hue = (x.p1.z.abs() * 360.0) % 360.0;
                hsv_to_rgb(hue, saturation, brightness)
            }
            ColorMode::Amplitude => {
                let hue = ((x.p1.x.abs() + x.p1.y.abs()) * 180.0) % 360.0;
                hsv_to_rgb(hue, saturation, brightness)
            }
            ColorMode::Position => {
                let norm_x = (crx1 / width as f64).abs();
                let norm_y = (cry1 / height as f64).abs();
                let hue = ((norm_x + norm_y) * 180.0) % 360.0;
                hsv_to_rgb(hue, saturation, brightness)
            }
            ColorMode::Noise => {
                let hue = ((x.radius / 1000.0) * 360.0) % 360.0;
                hsv_to_rgb(hue, saturation, brightness)
            }
        };

        context.set_line_width(0.1);
        context.set_source_rgba(r, g, b, 1.0);
        context.move_to(crx1, cry1);

        match method {
            Method::Arc => context.arc(crx1, cry1, x.radius, x.p1.z, x.p2.z),
            Method::Curve => context.curve_to(
                crx1,
                cry1,
                crx2,
                cry2,
                cx + x.p1.z * x.radius,
                cy + x.p2.z * x.radius,
            ),
            Method::Dot => {
                context.set_source_rgba(r, g, b, 1.0);
                if combine_dots {
                    context.rectangle(crx3, cry3, 0.5, 0.5);
                } else {
                    let size_1 = if size > 0. { x.p1.z.abs() * size } else { 1.0 };
                    let size_2 = if size > 0. { x.p2.z.abs() * size } else { 1.0 };
                    context.rectangle(crx1, cry1, size_1, size_1);
                    context.stroke().unwrap();
                    context.fill().unwrap();
                    context.set_source_rgba(r, g, b, 1.0);
                    context.rectangle(crx2, cry2, size_2, size_2);
                }
            }
            Method::Line => context.line_to(crx2, cry2),
        }
        context.stroke().unwrap();
    }
}
