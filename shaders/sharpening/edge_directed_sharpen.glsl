    {
        float eg2 = lgrad_x * lgrad_x + lgrad_y * lgrad_y;
        float ew = 0.5 * clamp(eg2 * 64.0, 0.0, 1.0);
        c = clamp(c + (c * 4.0 - cross_sum) * ew * 0.25, 0.0, 1.0);
    }
