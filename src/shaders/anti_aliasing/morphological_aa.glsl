    {
        vec3 ca = (tap_1_0 + tap_m1_0 + tap_0_1 + tap_0_m1) * 0.25;
        vec3 cd = (tap_1_1 + tap_m1_1 + tap_1_m1 + tap_m1_m1) * 0.25;
        vec3 dx_abs = abs(tap_1_0 - tap_m1_0);
        vec3 dy_abs = abs(tap_0_1 - tap_0_m1);
        float ce = (dx_abs.r + dx_abs.g + dx_abs.b + dy_abs.r + dy_abs.g + dy_abs.b) * 0.16666667;
        c = mix(c, (ca + cd) * 0.5, smoothstep(0.1, 0.2, ce) * 0.7);
    }
