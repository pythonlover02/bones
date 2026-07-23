    {
        vec2 nt = vec2(-lgrad_y, lgrad_x) * (1.5 * res_scale) * inv;
        float ng2 = lgrad_x * lgrad_x + lgrad_y * lgrad_y;
        c = mix(c, (texture(u_input, v_uv + nt).rgb +
                    texture(u_input, v_uv - nt).rgb + c) * 0.3333333,
                clamp(ng2 * 16.0, 0.0, 1.0));
    }
