    {
        const float MN_W_C   = 0.79012346;
        const float MN_W_AX4 = 0.19753088;
        const float MN_W_DI4 = 0.01234568;
        const float MN_W_NORM = 1.0 / (MN_W_C + MN_W_AX4 + MN_W_DI4);
        vec3 ms = (c * MN_W_C + d_cross_avg * MN_W_AX4 + d_corner_avg * MN_W_DI4) * MN_W_NORM;
        c = mix(c, mix(ms, history, 0.5), hist_valid);
    }
