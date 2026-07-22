    {
        float mx = max(c.r, max(c.g, c.b));
        float st = mx - min(c.r, min(c.g, c.b));
        float ir = step(abs(c.r - mx), 1e-5);
        float ig = step(abs(c.g - mx), 1e-5) * (1.0 - ir);
        float ib = step(abs(c.b - mx), 1e-5) * (1.0 - ir) * (1.0 - ig);
        c = c * (1.0 + ib * 0.3 * st);
    }
