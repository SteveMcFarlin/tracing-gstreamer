//! This example prepares a vp9 encoding pipeline, instrumented via tracing.
use gstreamer::TracerFactory;
use gstreamer::{traits::ElementExt, ClockTime, MessageView::*, State};

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;
use tracing_tracy::TracyLayer;

// use tracy_client::Span;

#[derive(Clone, Copy)]
pub struct CustomLayer;
impl<S> Layer<S> for CustomLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        println!("Got event!");
        println!("  level={:?}", event.metadata().level());
        println!("  target={:?}", event.metadata().target());
        println!("  name={:?}", event.metadata().name());
        for field in event.fields() {
            println!("  field={}", field.name());
        }
    }
}

fn main() {
    tracing::info!("Running example");
    let sub = tracing_subscriber::registry()
        //.with(tracing_subscriber::EnvFilter::from_default_env())
        // .with(TracyLayer::new())
        .with(tracing_tracy::TracyLayer::new());
    // .with(CustomLayer);

    //let _guard = tracing::subscriber::set_default(sub);
    tracing::subscriber::set_global_default(sub).expect("setting default subscriber failed");

    //tracing_subscriber::fmt::init();
    gstreamer::debug_remove_default_log_function();
    tracing_gstreamer::integrate_events();

    gstreamer::debug_set_default_threshold(gstreamer::DebugLevel::Memdump);
    gstreamer::init().expect("gst init");
    tracing_gstreamer::integrate_spans();

    let pipeline = gstreamer::parse_launch(
        r#"
        videotestsrc num-buffers=12000
        ! vp9enc
        ! webmmux name=mux
        ! fakesink sync=false

        audiotestsrc num-buffers=12000
        ! opusenc
        ! mux.
    "#,
    )
    .expect("construct the pipeline");

    let bus = pipeline.bus().expect("could not obtain the pipeline bus");
    pipeline
        .set_state(gstreamer::State::Playing)
        .expect("could not start the pipeline");
    loop {
        let msg = match bus.timed_pop(ClockTime::NONE) {
            None => break,
            Some(msg) => msg,
        };
        tracing::info!(message = "bus message", bus_message = ?msg);
        match msg.view() {
            Eos(_) => break,
            Error(e) => break tracing::error!("{}", e.error()),
            Warning(w) => tracing::warn!("{:?}", w),
            _ => {}
        }
    }
    pipeline
        .set_state(State::Null)
        .expect("could not stop the pipeline");
    println!("Done!")
}
