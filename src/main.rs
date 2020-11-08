use clap::{ Arg, App };
use std::path::Path;
use std::fs::File;
use gif::{ Frame, Encoder, Repeat, SetParameter };
use image::Rgba;

fn main() {
    let matches = App::new("gife")
        .settings(&[clap::AppSettings::ColoredHelp, clap::AppSettings::ColorAlways])
        .version("0.1")
        .author("Post-Rex <post-rex@pm.me>")
        .about("Magic gif encoding tool")
        .arg(Arg::with_name("verbose")
            .short("v")
            .long("verbose")
            .help("Prints current steps and progress")
        ).arg(Arg::with_name("output")
            .short("o")
            .long("output")
            .help("Path to where the fill shall be written to")
            .takes_value(true)
            .required(true)
        ).arg(Arg::with_name("overwrite")
            .short("O")
            .long("overwrite")
            .help("Overwrites the output file if it already exists")
        ).arg(Arg::with_name("delay")
            .long("delay")
            .short("d")
            .takes_value(true)
            .help("Frame delay in units of 10 ms\nCan be used as alternative to --fps")
            .required_unless("fps")
        ).arg(Arg::with_name("fps")
            .long("fps")
            .short("f")
            .takes_value(true)
            .help("The targeted amount of frames per second\nMaximum: 100\nCan be used as alternative to --delay")
            .required_unless("delay")
        ).arg(Arg::with_name("allow-transparancy")
            .short("t")
            .long("preserve-transparancy")
            .help("Preserve transparancy if found in image")
        ).arg(Arg::with_name("width")
            .long("width")
            .takes_value(true)
            .help("Sets the width of each frame") // , this does not effect height unless --preserve-aspect-ratio is set
        ).arg(Arg::with_name("height")
            .long("height")
            .takes_value(true)
            .help("Sets the height of each frame") // , this does not effect width unless --preserve-aspect-ratio is set
        )/*.arg(Arg::with_name("preserve-aspect-ratio")
            .long("preserve-aspect-ratio")
            .help("Preserves the aspect ratio of a frame when --width or --height is set.\nTakes: true & false")
            .takes_value(true)
        )*/.usage("gife [FLAGS] -o <output> [OPTIONS] <FILE> [<FILE>]")
        .arg(Arg::with_name("from-files")
            .takes_value(true)
            .multiple(true)
        )
        .get_matches();
    
    let verbose = matches.is_present("verbose");
    let mut delay = match matches.value_of("fps").or(matches.value_of("delay")).unwrap().parse::<u16>() {
        Ok(x) => x,
        Err(_) => error("--fps or --delay needs to be a number".to_string())
    };
    if matches.is_present("fps") {
        delay = 100 / delay;
    }
    if matches.is_present("width") ^ matches.is_present("height") {
        error::<()>("You can only set either both or none of --width and --height".to_string());
    }
    let mut width: u32 = 0;
    let mut height: u32 = 0;
    if matches.is_present("width") && matches.is_present("height") {
        width = match matches.value_of("width").unwrap().to_string().parse() {
            Ok(x) => x,
            Err(_) => error("--width needs to be a number".to_string())
        };
        height = match matches.value_of("height").unwrap().to_string().parse() {
            Ok(x) => x,
            Err(_) => error("--height needs to be a number".to_string())
        };
    }
    let output = matches.value_of("output").unwrap();
    if !matches.is_present("overwrite") && Path::new(output).exists() {
        error::<()>("The file already exists, to overwrite a file use the flag -O".to_string());
    }
    if verbose {
        print_logo();
    }

    if verbose { println!("Checking files..."); }
    let files = matches.values_of("from-files").unwrap();
    for file in files.clone() {
        if !Path::new(file).exists() {
            error::<()>(format!("File {} does not exist", file));
        }
    }
    let allow_transparancy = matches.is_present("allow-transparancy");
    if verbose { println!("Creating output file..."); }
    let file = File::create(output).unwrap();
    let file_ = files.clone().next().unwrap();
    let img_ = match image::open(file_) {
        Ok(x) => x,
        Err(_) => error(format!("{} is not a known image file", file_))
    };
    let temp_img = img_.to_rgba();
    let empty: [u8; 256] = unsafe { std::mem::zeroed() };
    let mut encoder = Encoder::new(file, temp_img.width() as u16, temp_img.height() as u16, &empty).unwrap();
    for file in files {
        if verbose { println!("Reading image {}", file); }
        let mut img = match image::open(file) {
            Ok(x) => x,
            Err(_) => error(format!("{} is not a known image file", file))
        };
        if width != 0  {
            img = img.resize(u32::from(width), u32::from(height), image::imageops::FilterType::Lanczos3);
        }
        let rgba = img.to_rgba();
        if width == 0 {
            width = rgba.width();
            height = rgba.height();
        }
        let mut data = Vec::with_capacity((width * height * 4) as usize);
        for y in 0..rgba.height() {
            for x in 0..rgba.width() {
                let pixel: &Rgba<u8> = rgba.get_pixel(x, y);
                let alpha = if allow_transparancy { pixel[3] } else { 255 };
                data.push(if alpha == 0 { 0 } else { pixel[0] });
                data.push(if alpha == 0 { 0 } else { pixel[1] });
                data.push(if alpha == 0 { 0 } else { pixel[2] });
                data.push(alpha);
            }
        }
        if verbose { println!("Processing file..."); }
        let mut frame = Frame::from_rgba(width as u16, height as u16, &mut data[..(width * height * 4) as usize]);
        frame.delay = delay;
        frame.dispose = gif::DisposalMethod::Background;
        if verbose { println!("Writing frame..."); }
        encoder.write_frame(&frame).unwrap();
    }
    encoder.set(Repeat::Infinite).unwrap();
    println!("{green}Encoding complete!{reset}", green = "\033[32m", reset = "\033[0m");
}

fn print_logo() {
    println!(
"{red}   ________{yellow}.__ {green} _____{blue}___________
{red}  /  _____/{yellow}|__|{green}/ ____{blue}\\_   _____/
{red} /   \\  ___{yellow}|  \\{green}   __\\ {blue}|    __)_
{red} \\    \\_\\  \\{yellow}  |{green}|  |   {blue}|        \\
{red}  \\______  /{yellow}__|{green}|__|  {blue}/_______  /
{red}         \\/                  {blue}\\/  {reset}v.1\n",
        red = "\033[31m",
        yellow = "\033[33m",
        green = "\033[32m",
        blue = "\033[34m",
        reset = "\033[0m"
    );
}

fn error<T>(message: String) -> T {
    println!("{red}[ERROR] {message}{reset}", message = message, red = "\033[31m", reset = "\033[0m");
    std::process::exit(1);
}
