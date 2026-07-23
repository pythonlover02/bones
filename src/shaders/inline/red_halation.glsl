    {
        const float HAL_THR = 0.6;
        float hal_sum_r = tap_1_1.r + tap_m1_1.r + tap_1_m1.r + tap_m1_m1.r;
        c.r = c.r + max(hal_sum_r * 0.25 - HAL_THR, 0.0) * 0.5;
    }
