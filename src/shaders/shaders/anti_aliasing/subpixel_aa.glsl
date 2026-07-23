    {
        float sh = abs(tap_1_0.g - c.g) + abs(tap_m1_0.g - c.g);
        float sv = abs(tap_0_1.g - c.g) + abs(tap_0_m1.g - c.g);
        float se = max(sh, sv);
        vec3 sc2 = mix((tap_1_0 + tap_m1_0) * 0.5,
                       (tap_0_1 + tap_0_m1) * 0.5,
                       step(sv, sh));
        vec3 sd2 = (tap_1_1 + tap_m1_1 + tap_1_m1 + tap_m1_m1) * 0.25;
        c = mix(c, mix(sc2, sd2, 0.3), min(smoothstep(0.1, 0.2, se), 0.75));
    }
