    {
        vec3 tx = max(c, ZERO3);
        c = clamp(tx / (tx + vec3(0.155)) * 1.19, 0.0, 1.0);
    }
