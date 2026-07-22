    {
        float kx = min(c.r, min(c.g, c.b));
        float ko = mix(0.04, kx - 6.25 * kx * kx, step(kx, 0.08));
        c -= ko;
        float kp = max(c.r, max(c.g, c.b));
        float knp = 1.0 - 0.0576 / max(kp + 0.24 - 0.76, 0.0001);
        float kgate = step(0.76, kp);
        c *= mix(1.0, knp / max(kp, 0.0001), kgate);
        float kg = 1.0 - 1.0 / max(0.15 * (kp - knp) + 1.0, 0.0001);
        c = clamp(mix(c, vec3(knp), kgate * kg), 0.0, 1.0);
    }
