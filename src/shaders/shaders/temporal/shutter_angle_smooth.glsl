    {
        vec3 sd = c - history;
        float sm = clamp(dot(sd, sd) * 30.0, 0.0, 1.0);
        float sw = 0.5 * (1.0 - sm) + 0.15 * sm;
        c = mix(c, mix(c, history, sw), hist_valid);
    }
