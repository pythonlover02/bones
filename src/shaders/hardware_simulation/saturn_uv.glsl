    {
        float sat_ps = 2.0 * res_scale;
        v_uv = (floor(v_uv * u_resolution / sat_ps) + 0.5) * sat_ps * inv;
    }
