use futures::stream::Stream;

pub trait WorkloadListener {
    fn getStream(&self) -> Box<dyn Stream<Item = String>>;
    //                                              ^TBD
}