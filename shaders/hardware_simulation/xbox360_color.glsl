    {
        float x3_s1 = 1.0 - smoothstep(0.0, 0.003, abs(v_uv.y - 0.333));
        float x3_s2 = 1.0 - smoothstep(0.0, 0.003, abs(v_uv.y - 0.667));
        c = c + vec3(0.015) * (x3_s1 + x3_s2);
        float x3_lum = dot(c, LUMA_BT601);
        float x3_steps = mix(256.0, 24.0, smoothstep(0.5, 0.9, x3_lum));
        float x3_qlum = floor(x3_lum * x3_steps + 0.5) / x3_steps;
        c = c * (x3_qlum / max(x3_lum, 0.0001));
        c = c + vec3(0.04) * (ONE3 - c);
        c = c * vec3(1.03, 1.01, 0.96);
        float x3_g = dot(c, LUMA_BT601);
        c = mix(vec3(x3_g), c, 0.92);
        c = c + vec3(smoothstep(0.55, 0.85, x3_g)) * 0.06;
    }
