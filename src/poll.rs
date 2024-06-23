use arrayvec::ArrayVec;
use rustix::event::{poll as rustix_poll, PollFd};

/// Block until any of the [PollFd]s is satisfied. Returns an array with the [PollFd]s that had
/// events (the result array's order is equal to the input order). If any [PollFd] is [None], it is
/// ignored and its result is `false`.
pub fn poll<'fd, const C: usize>(
    mut poll_fds: [Option<PollFd<'fd>>; C],
) -> std::io::Result<[bool; C]> {
    let mut polls = ArrayVec::<_, C>::new();
    let mut poll_nums = ArrayVec::<_, C>::new();

    for idx in 0..C {
        if let Some(poll_fd) = poll_fds[idx].take() {
            polls.push(poll_fd);
            poll_nums.push(idx);
        }
    }

    rustix_poll(&mut polls, 0)?;

    let mut result = [false; C];
    for (poll, poll_num) in polls.into_iter().zip(poll_nums) {
        result[poll_num] = !poll.revents().is_empty();
    }

    Ok(result)
}
