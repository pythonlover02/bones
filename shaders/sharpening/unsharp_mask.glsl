    {
        vec3 ub = (c * 4.0 + cross_sum + corner_sum * 0.5) * 0.1;
        c = c + (c - ub) * 0.5;
    }
