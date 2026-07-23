    {
        float n64_ps = 1.5 * res_scale;
        v_uv = (floor(v_uv * u_resolution / n64_ps) + 0.5) * n64_ps * inv;
    }
