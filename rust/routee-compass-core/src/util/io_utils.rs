use kdam::Bar;
use kdam::BarExt;

pub type ProgressBarOp<'a, R, E> = Box<dyn FnMut(Box<dyn FnMut()>) -> Result<R, E> + 'a>;

/// runs some closure expression which accepts a zero-arity function
/// callback. the callback should be invoked at the inner loop of
/// whatever iterative operation takes place, as it will trigger a
/// progress bar update. the callback should be called exactly "count" times.
pub fn with_progress_bar<R, E>(
    mut closure: ProgressBarOp<'_, R, E>,
    error: Box<dyn Fn(String) -> E>,
    count: usize,
    message: String,
    animation: String,
) -> Result<R, E> {
    let mut pb = Bar::builder()
        .total(count)
        .animation(animation.as_str())
        .desc(message)
        .build()
        .map_err(error)?;
    let cb = Box::new(move || {
        let _ = pb.update(1);
    });

    let result = closure(cb);

    println!(); // create a newline once the progress bar is complete

    result
}
