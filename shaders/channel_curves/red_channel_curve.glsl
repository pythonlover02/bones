    {
        float rv = c.r * c.r * (3.0 - 2.0 * c.r);
        c.r = mix(c.r, rv, 0.5);
    }
