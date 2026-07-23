    {
        vec3 dd_t = abs(c - history);
        float dm_t = max(dd_t.r, max(dd_t.g, dd_t.b));
        vec3 c1 = mix(c, history, 0.5 * (1.0 - step(0.15, dm_t)));
        vec3 td_t = history - c1;
        float tw_t = 0.3 * (1.0 - clamp(dot(td_t, td_t) * 10.0, 0.0, 1.0));
        vec3 c2 = clamp(mix(c1, history + td_t * tw_t, tw_t), 0.0, 1.0);
        c = mix(c, c2, hist_valid);
    }
