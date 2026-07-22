    {
        vec3 e00 = tap_m1_m1 - c; float w00 = 1.0 / (1.0 + dot(e00, e00) * 100.0);
        vec3 e10 = tap_0_m1  - c; float w10 = 1.0 / (1.0 + dot(e10, e10) * 100.0);
        vec3 e20 = tap_1_m1  - c; float w20 = 1.0 / (1.0 + dot(e20, e20) * 100.0);
        vec3 e01 = tap_m1_0  - c; float w01 = 1.0 / (1.0 + dot(e01, e01) * 100.0);
        vec3 e21 = tap_1_0   - c; float w21 = 1.0 / (1.0 + dot(e21, e21) * 100.0);
        vec3 e02 = tap_m1_1  - c; float w02 = 1.0 / (1.0 + dot(e02, e02) * 100.0);
        vec3 e12 = tap_0_1   - c; float w12 = 1.0 / (1.0 + dot(e12, e12) * 100.0);
        vec3 e22 = tap_1_1   - c; float w22 = 1.0 / (1.0 + dot(e22, e22) * 100.0);
        vec3 ds = tap_m1_m1*w00 + tap_0_m1*w10 + tap_1_m1*w20 + tap_m1_0*w01 + c
                + tap_1_0*w21 + tap_m1_1*w02 + tap_0_1*w12 + tap_1_1*w22;
        float dw = w00 + w10 + w20 + w01 + 1.0 + w21 + w02 + w12 + w22;
        c = mix(c, ds / max(dw, 0.0001), 0.6);
    }
