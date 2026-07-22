    {
        vec3 md = abs(c - history);
        float mm = clamp(max(md.r, max(md.g, md.b)) * 8.0, 0.0, 1.0);
        c = mix(c, mix(history, c, mix(1.0, 0.5, mm)), hist_valid);
    }
