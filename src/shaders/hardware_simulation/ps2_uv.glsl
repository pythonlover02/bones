    {
        float ps2_ps = 1.8 * res_scale;
        v_uv = (floor(v_uv * u_resolution / ps2_ps) + 0.5) * ps2_ps * inv;
    }
