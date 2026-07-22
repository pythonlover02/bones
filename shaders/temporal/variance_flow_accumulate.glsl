    {
        float dl = dot(c, LUMA_AVG);
        float dx = lgrad_x * 0.5;
        float dy = lgrad_y * 0.5;
        float dt = dl - dot(history, LUMA_AVG);
        float dd2 = dx * dx + dy * dy + 0.001;
        vec2 df = clamp(vec2(dx, dy) * (-dt / dd2),
                        vec2(-4.0 * res_scale), vec2(4.0 * res_scale));
        vec3 dw = texture(u_history, v_uv + df * inv).rgb;
        vec3 dm1 = (c + cross_sum) * 0.2;
        vec3 dm2 = (c * c + tap_1_0 * tap_1_0 + tap_m1_0 * tap_m1_0 +
                    tap_0_1 * tap_0_1 + tap_0_m1 * tap_0_m1) * 0.2;
        vec3 ds_ = sqrt(max(dm2 - dm1 * dm1, ZERO3));
        vec3 dlo = grade(dm1 - ds_ * 1.25, v_uv, frag_coord, res_scale);
        vec3 dhi = grade(dm1 + ds_ * 1.25, v_uv, frag_coord, res_scale);
        vec3 dc_ = clamp(dw, min(dlo, dhi), max(dlo, dhi));
        vec3 dcd = abs(dw - dc_);
        float dca = max(dcd.r, max(dcd.g, dcd.b));
        float dmc = 1.0 - clamp(dot(df, df) * 144.0 / max(res_scale * res_scale, 0.0001), 0.0, 1.0);
        float dlc = 1.0 - clamp(abs(dot(c, LUMA_BT709) - dot(dc_, LUMA_BT709)) * 8.0, 0.0, 1.0);
        float dcf = dmc * dlc * (1.0 - clamp(dca * 8.0, 0.0, 1.0));
        float vw = mix(0.05, 0.85, dcf * dcf);
        c = mix(c, mix(c, dc_, vw), hist_valid);
    }
