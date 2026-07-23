    {
        float mx = max(c.r, max(c.g, c.b));
        float st = mx - min(c.r, min(c.g, c.b));
        c = c * (1.0 + step(abs(c.r - mx), 1e-5) * 0.3 * st);
    }
