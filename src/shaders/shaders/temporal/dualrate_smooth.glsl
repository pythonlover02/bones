    {
        vec3 rm = (c + cross_sum) * 0.2;
        vec3 r0 = c - rm;
        vec3 r1 = tap_1_0 - rm;
        vec3 r2 = tap_m1_0 - rm;
        vec3 r3 = tap_0_1 - rm;
        vec3 r4 = tap_0_m1 - rm;
        float rv = (dot(r0, r0) + dot(r1, r1) + dot(r2, r2) + dot(r3, r3) + dot(r4, r4)) * 0.2;
        float rw = mix(0.7, 0.1, clamp(rv * 50.0, 0.0, 1.0));
        c = mix(c, mix(c, history, rw), hist_valid);
    }
