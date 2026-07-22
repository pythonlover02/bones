    {
        float tri = abs(fract(v_uv.y * u_resolution.y * 0.5) - 0.5) * 2.0;
        c = c * (1.0 - 0.3 * tri);
    }
