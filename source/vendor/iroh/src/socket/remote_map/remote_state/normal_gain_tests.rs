//! Exercise the production measurement-to-move boundary with actual ACK time windows.
use super::*;

const MS: fn(u64) -> Duration = Duration::from_millis;

fn road(port: u16) -> transports::FourTuple {
    transports::FourTuple::Ip {
        remote: std::net::SocketAddr::from(([127, 0, 0, 1], port)),
        local: None,
    }
}

fn observe(
    measured: &mut Measured,
    key: ConnId,
    road: &transports::FourTuple,
    at: Instant,
    rtt: Duration,
) {
    let old = measured.rtts.remove(&(key, PathId::ZERO));
    let mut stats = PathStats::default();
    stats.rtt = rtt;
    stats.latest_rtt = rtt;
    stats.rtt_samples = old.as_ref().map_or(1, |r| r.samples + 1);
    stats.latest_rtt_sent = Some((at - rtt).into_std());
    stats.latest_rtt_received = Some(at.into_std());
    let reading = match old {
        Some(mut reading) => {
            assert!(reading.observe(&stats, at));
            reading
        }
        None => Reading::new(road.clone(), &stats, at),
    };
    measured.rtts.insert((key, PathId::ZERO), reading);
}

fn decision(
    t: Instant,
) -> (
    Measured,
    Moving,
    transports::FourTuple,
    transports::FourTuple,
) {
    let (from, to) = (road(4701), road(4702));
    let mut measured = Measured::default();
    observe(&mut measured, ConnId(0), &from, t - MS(50), MS(330));
    observe(&mut measured, ConnId(1), &to, t - MS(50), MS(300));
    let mut moving = Moving::default();
    assert!(matches!(
        moving.measured_step(&from, &to, t, &measured),
        Step::Hold { .. }
    ));
    (measured, moving, from, to)
}

fn window(
    measured: &mut Measured,
    from: &transports::FourTuple,
    to: &transports::FourTuple,
    t: Instant,
    a: [u64; 4],
    b: [u64; 4],
) {
    for (index, (a, b)) in a.into_iter().zip(b).enumerate() {
        let at = t + MS(900 + 300 * index as u64);
        observe(measured, ConnId(0), from, at, MS(a));
        observe(measured, ConnId(1), to, at, MS(b));
    }
    for reading in measured.rtts.values_mut() {
        reading.expire(t + CONFIRM_FOR);
    }
}

#[test]
fn overlapping_recent_round_trips_keep_the_healthy_current_road() {
    let t = Instant::now();
    let (mut measured, mut moving, from, to) = decision(t);
    window(
        &mut measured,
        &from,
        &to,
        t,
        [280, 340, 320, 330],
        [290, 310, 300, 300],
    );
    assert_eq!(measured.road(&from).unwrap().0, MS(330));
    assert_eq!(measured.road(&to).unwrap().0, MS(300));
    assert_ne!(
        moving.measured_step(&from, &to, t + CONFIRM_FOR, &measured),
        Step::Go,
        "a lower median inside the current road's observed variation is not a confirmed gain"
    );
    assert_eq!(
        moving.measured_step(&from, &to, t + CONFIRM_MAX, &measured),
        Step::Rest,
        "a rejected normal move must still stop at the existing finite budget"
    );
}

#[test]
fn clearly_separated_recent_round_trips_allow_a_normal_move() {
    let t = Instant::now();
    let (mut measured, mut moving, from, to) = decision(t);
    window(
        &mut measured,
        &from,
        &to,
        t,
        [105, 100, 110, 95],
        [65, 60, 55, 60],
    );
    assert_eq!(
        moving.measured_step(&from, &to, t + CONFIRM_FOR, &measured),
        Step::Go
    );
}

#[test]
fn a_far_faster_fresh_candidate_keeps_the_existing_fast_escape() {
    let t = Instant::now();
    let (mut measured, mut moving, from, to) = decision(t);
    window(
        &mut measured,
        &from,
        &to,
        t,
        [15, 400, 300, 330],
        [20, 22, 19, 20],
    );
    assert_eq!(
        moving.measured_step(&from, &to, t + MS(1900), &measured),
        Step::Go
    );
}

#[test]
fn a_current_road_slower_than_confirmation_does_not_block_on_its_sample() {
    let t = Instant::now();
    let (mut measured, mut moving, from, to) = decision(t);
    measured.rtts.remove(&(ConnId(0), PathId::ZERO));
    observe(&mut measured, ConnId(0), &from, t - MS(1), MS(2500));
    for at in [1500, 1800] {
        observe(&mut measured, ConnId(1), &to, t + MS(at), MS(1400));
    }
    assert_eq!(
        moving.measured_step(&from, &to, t + CONFIRM_FOR, &measured),
        Step::Go
    );
}

#[test]
fn a_silent_current_road_still_leaves_on_one_new_backup_response() {
    let t = Instant::now();
    let (_, mut moving, _, to) = decision(t);
    assert_eq!(
        moving.leave(&to, t + MS(250), t, false, |after| t + MS(100) > after),
        Step::Go
    );
}
