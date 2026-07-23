    {
        const vec3 THR_BLOOM = vec3(0.7);
        vec3 bs = max(tap_1_0 - THR_BLOOM, ZERO3) * 2.0 +
                  max(tap_m1_0 - THR_BLOOM, ZERO3) * 2.0 +
                  max(tap_0_1 - THR_BLOOM, ZERO3) * 2.0 +
                  max(tap_0_m1 - THR_BLOOM, ZERO3) * 2.0 +
                  max(tap_1_1 - THR_BLOOM, ZERO3) +
                  max(tap_m1_1 - THR_BLOOM, ZERO3) +
                  max(tap_1_m1 - THR_BLOOM, ZERO3) +
                  max(tap_m1_m1 - THR_BLOOM, ZERO3);
        c = c + bs * 0.05;
    }
