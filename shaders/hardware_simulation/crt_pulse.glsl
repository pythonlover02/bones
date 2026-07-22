    {
        float crt_bright = 1.0 + fast_sin(u_time * 1.7) * 0.04 + fast_sin(u_time * 0.3) * 0.02;
        c = c * crt_bright;
    }
