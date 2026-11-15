use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::thread::JoinHandle;

use parking_lot::Mutex;
use pipewire as pw;
use pw::properties::properties;
use pw::spa;
use pw::spa::pod::Pod;
use pw::stream::{StreamFlags, StreamListener, StreamRc};

use crate::RATE;
use crate::mixer::Mixer;

const MAX_FRAMES: usize = 8192;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Device {
    pub id: u32,
    pub name: String,
    pub description: String,
}

enum Message {
    Target(Option<String>),
    Quit,
}

/// The PipeWire stream the mix plays through, and the list of places it can play.
pub struct Output {
    sender: pw::channel::Sender<Message>,
    thread: Option<JoinHandle<()>>,
}

impl Output {
    pub fn start(
        mixer: Mixer,
        target: Option<String>,
        on_devices: impl Fn(Vec<Device>) + Send + 'static,
    ) -> Output {
        let (sender, receiver) = pw::channel::channel();
        let mixer = Arc::new(Mutex::new(mixer));
        let thread = std::thread::Builder::new()
            .name("attacca-output".into())
            .spawn(move || {
                if let Err(error) = run(mixer, target, receiver, on_devices) {
                    eprintln!("attacca: audio output stopped: {error}");
                }
            })
            .expect("the output thread starts");
        Output {
            sender,
            thread: Some(thread),
        }
    }

    pub fn set_target(&self, target: Option<String>) {
        let _ = self.sender.send(Message::Target(target));
    }
}

impl Drop for Output {
    fn drop(&mut self) {
        let _ = self.sender.send(Message::Quit);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

type Playing = (StreamRc, StreamListener<Vec<f32>>);

fn run(
    mixer: Arc<Mutex<Mixer>>,
    target: Option<String>,
    receiver: pw::channel::Receiver<Message>,
    on_devices: impl Fn(Vec<Device>) + 'static,
) -> Result<(), pw::Error> {
    pw::init();
    let mainloop = pw::main_loop::MainLoopRc::new(None)?;
    let context = pw::context::ContextRc::new(&mainloop, None)?;
    let core = context.connect_rc(None)?;
    let registry = core.get_registry_rc()?;

    let devices = Rc::new(RefCell::new(Vec::<Device>::new()));
    let on_devices = Rc::new(on_devices);
    let _devices_listener = registry
        .add_listener_local()
        .global({
            let devices = Rc::clone(&devices);
            let on_devices = Rc::clone(&on_devices);
            move |global| {
                let Some(props) = global.props else { return };
                if props.get(*pw::keys::MEDIA_CLASS) != Some("Audio/Sink") {
                    return;
                }
                let name = props
                    .get(*pw::keys::NODE_NAME)
                    .unwrap_or_default()
                    .to_string();
                let description = props
                    .get(*pw::keys::NODE_DESCRIPTION)
                    .map_or_else(|| name.clone(), str::to_string);
                devices.borrow_mut().push(Device {
                    id: global.id,
                    name,
                    description,
                });
                on_devices(devices.borrow().clone());
            }
        })
        .global_remove({
            let devices = Rc::clone(&devices);
            let on_devices = Rc::clone(&on_devices);
            move |id| {
                let before = devices.borrow().len();
                devices.borrow_mut().retain(|device| device.id != id);
                if devices.borrow().len() != before {
                    on_devices(devices.borrow().clone());
                }
            }
        })
        .register();

    let playing: Rc<RefCell<Option<Playing>>> = Rc::new(RefCell::new(Some(connect(
        &core,
        target.as_deref(),
        &mixer,
    )?)));
    let _receiver = receiver.attach(mainloop.loop_(), {
        let mainloop = mainloop.clone();
        let playing = Rc::clone(&playing);
        move |message| match message {
            Message::Target(target) => {
                if let Some((stream, _)) = playing.borrow_mut().take() {
                    let _ = stream.disconnect();
                }
                match connect(&core, target.as_deref(), &mixer) {
                    Ok(stream) => *playing.borrow_mut() = Some(stream),
                    Err(error) => eprintln!("attacca: could not switch output: {error}"),
                }
            }
            Message::Quit => mainloop.quit(),
        }
    });
    mainloop.run();
    Ok(())
}

fn connect(
    core: &pw::core::CoreRc,
    target: Option<&str>,
    mixer: &Arc<Mutex<Mixer>>,
) -> Result<Playing, pw::Error> {
    let mut props = properties! {
        *pw::keys::MEDIA_TYPE => "Audio",
        *pw::keys::MEDIA_ROLE => "Music",
        *pw::keys::MEDIA_CATEGORY => "Playback",
        *pw::keys::AUDIO_CHANNELS => "2",
        *pw::keys::NODE_NAME => "attacca",
        *pw::keys::NODE_DESCRIPTION => "Attacca",
        *pw::keys::APP_NAME => "Attacca",
        *pw::keys::APP_ID => "al.kaleid.attacca",
        *pw::keys::APP_ICON_NAME => "al.kaleid.attacca",
        *pw::keys::NODE_LATENCY => format!("1024/{RATE}"),
    };
    if let Some(target) = target {
        props.insert("target.object", target);
    }
    let stream = StreamRc::new(core.clone(), "Attacca", props)?;
    let mixer = Arc::clone(mixer);
    let listener = stream
        .add_local_listener_with_user_data(vec![0.0f32; MAX_FRAMES * 2])
        .process(move |stream, scratch| {
            let Some(mut buffer) = stream.dequeue_buffer() else {
                return;
            };
            let data = &mut buffer.datas_mut()[0];
            let stride = std::mem::size_of::<f32>() * 2;
            let Some(bytes) = data.data() else { return };
            let frames = (bytes.len() / stride).min(MAX_FRAMES);
            let samples = &mut scratch[..frames * 2];
            match mixer.try_lock() {
                Some(mut mixer) => mixer.render(samples),
                None => samples.fill(0.0),
            }
            for (chunk, sample) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(samples.iter()) {
                chunk.copy_from_slice(&sample.to_le_bytes());
            }
            let chunk = data.chunk_mut();
            *chunk.offset_mut() = 0;
            *chunk.stride_mut() = stride as i32;
            *chunk.size_mut() = (stride * frames) as u32;
        })
        .register()?;

    let mut info = spa::param::audio::AudioInfoRaw::new();
    info.set_format(spa::param::audio::AudioFormat::F32LE);
    info.set_rate(RATE);
    info.set_channels(2);
    let mut position = [0; spa::param::audio::MAX_CHANNELS];
    position[0] = spa::sys::SPA_AUDIO_CHANNEL_FL;
    position[1] = spa::sys::SPA_AUDIO_CHANNEL_FR;
    info.set_position(position);
    let values = spa::pod::serialize::PodSerializer::serialize(
        std::io::Cursor::new(Vec::new()),
        &spa::pod::Value::Object(spa::pod::Object {
            type_: spa::sys::SPA_TYPE_OBJECT_Format,
            id: spa::sys::SPA_PARAM_EnumFormat,
            properties: info.into(),
        }),
    )
    .expect("the audio format serializes")
    .0
    .into_inner();
    let mut params = [Pod::from_bytes(&values).expect("the audio format is a pod")];
    stream.connect(
        spa::utils::Direction::Output,
        None,
        StreamFlags::AUTOCONNECT | StreamFlags::MAP_BUFFERS | StreamFlags::RT_PROCESS,
        &mut params,
    )?;
    Ok((stream, listener))
}
