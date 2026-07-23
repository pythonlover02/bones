    {
        vec3 vm1 = (c + cross_sum + corner_sum) * 0.1111111;
        vec3 vm2 = (c * c + tap_1_0 * tap_1_0 + tap_m1_0 * tap_m1_0 +
                    tap_0_1 * tap_0_1 + tap_0_m1 * tap_0_m1 +
                    tap_1_1 * tap_1_1 + tap_m1_1 * tap_m1_1 +
                    tap_1_m1 * tap_1_m1 + tap_m1_m1 * tap_m1_m1) * 0.1111111;
        vec3 vsd = sqrt(max(vm2 - vm1 * vm1, ZERO3));
        vec3 vlo = grade(vm1 - vsd, v_uv, frag_coord, res_scale);
        vec3 vhi = grade(vm1 + vsd, v_uv, frag_coord, res_scale);
        vec3 vh = clamp(history, min(vlo, vhi), max(vlo, vhi));
        c = mix(c, mix(c, vh, 0.6), hist_valid);
    }
