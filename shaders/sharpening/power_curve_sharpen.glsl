    {
        vec3 pd = c - cross_avg;
        vec3 ap = abs(pd);
        c = c + sign(pd) * (ap / (ap + vec3(0.3))) * 0.65;
    }
