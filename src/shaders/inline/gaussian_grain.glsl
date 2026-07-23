    {
        float g1 = max(hash21(v_uv + vec2(u_time * 10.0)), 0.0001);
        float g2 = hash21(v_uv + vec2(u_time * 10.0 + 7.31));
        float gg = sqrt(-2.0 * log(g1)) * fast_cos(6.2831853 * g2);
        c = c + vec3(gg) * 0.05;
    }
