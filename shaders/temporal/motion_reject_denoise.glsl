    {
        vec3 td = abs(c - history);
        float tm = max(td.r, max(td.g, td.b));
        c = mix(c, mix(c, history, 0.8 * (1.0 - clamp(tm * 8.0, 0.0, 1.0))), hist_valid);
    }
