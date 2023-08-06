use std::time::Duration;

/// Создает задержку между указанными значениями
pub fn random_between( min:Duration, max:Duration ) -> Duration {
    let (min,max) = if min <= max {
        (min,max)
    } else {
        (max,min)
    };

    let rand_u32 = rand::random::<u32>();
    let rand_f64_0_1 : f64 = (rand_u32 as f64) / (u32::MAX as f64);

    let mic0 = min.as_micros();
    let mic1 = max.as_micros();

    let dur_disp = mic1.max(mic0) - mic1.min(mic0);
    let dur_disp = ((dur_disp as f64) * rand_f64_0_1) as u128;

    let dur = dur_disp + mic0.min(mic1);
    let dur = Duration::from_micros(dur as u64);

    dur
}