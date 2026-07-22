    {
        float vrip_arg = v_uv.y * 4.77464829;
        float vrip = (abs(fract(vrip_arg) - 0.5) * 4.0 - 1.0) * 1.5 * res_scale * inv.x;
        v_uv.x = v_uv.x + vrip;
    }
