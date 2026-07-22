    {
        float om = 4.0 * res_scale;
        float ol = dot(c, LUMA_AVG);
        float ot = ol - dot(history, LUMA_AVG);
        float od = lgrad_x * lgrad_x * 0.25 + lgrad_y * lgrad_y * 0.25 + 0.001;
        vec2 of2 = clamp(vec2(lgrad_x * 0.5, lgrad_y * 0.5) * (-ot / od), vec2(-om), vec2(om));
        vec3 hs_out = mix(c, texture(u_history, v_uv + of2 * inv).rgb, 0.5);
        c = mix(c, hs_out, hist_valid);
    }
