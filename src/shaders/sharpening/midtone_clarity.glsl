    {
        float ml = dot(c, LUMA_AVG);
        float mm = 1.0 - abs(ml * 2.0 - 1.0);
        c = c + (c - cross_avg) * 0.5 * mm;
    }
