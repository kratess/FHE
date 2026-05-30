2026-05-30T12:46:21.407981Z DEBUG fhe_vdaf_2: [split_value] value_scaled=102 max_per_agg=100 seed=42
2026-05-30T12:46:21.408074Z DEBUG fhe_vdaf_2: [split_value] parts=[48, 54]
2026-05-30T12:46:22.144850Z DEBUG fhe_vdaf_2: [ct] client_ct slots=72 decoded=[0, 0, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0]
2026-05-30T12:46:22.360136Z DEBUG fhe_vdaf_2: [ct] client_ct slots=72 decoded=[0, 1, 1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0]
2026-05-30T12:46:22.360329Z DEBUG fhe_vdaf_2: [split_value] value_scaled=98 max_per_agg=100 seed=42
2026-05-30T12:46:22.360366Z DEBUG fhe_vdaf_2: [split_value] parts=[46, 52]
2026-05-30T12:46:23.111329Z DEBUG fhe_vdaf_2: [ct] client_ct slots=72 decoded=[0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0]
2026-05-30T12:46:23.280834Z DEBUG fhe_vdaf_2: [ct] client_ct slots=72 decoded=[0, 0, 1, 0, 1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0]
aggregator 0: aggregating 2 encrypted shard(s)
aggregator 0: produced encrypted aggregate share
2026-05-30T12:46:45.853253Z DEBUG fhe_vdaf_2: [ct] aggregator_0_aggregate_share slots=72 decoded=[0, 1, 1, 1, 1, 2, 0, 0, 1, 1, 1, 0, 2, 0, 2, 1, 2, 0, 2, 2, 0, 0, 1, 0, 1, 0, 1, 0, 2, 0, 1, 1, 1, 2, 1, 1, 2, 1, 1, 1, 1, 1, 2, 0, 1, 2, 2, 1, 1, 2, 1, 1, 2, 2, 2, 2, 0, 1, 2, 0, 0, 1, 1, 2, 2, 1, 2, 1, 1, 1, 0, 2]
aggregator 1: aggregating 2 encrypted shard(s)
aggregator 1: produced encrypted aggregate share
2026-05-30T12:47:11.735516Z DEBUG fhe_vdaf_2: [ct] aggregator_1_aggregate_share slots=72 decoded=[0, 1, 2, 0, 2, 2, 0, 0, 1, 1, 1, 0, 2, 0, 2, 1, 2, 0, 2, 2, 0, 0, 1, 0, 1, 0, 1, 0, 2, 0, 1, 1, 1, 2, 1, 1, 2, 1, 1, 1, 1, 1, 2, 0, 1, 2, 2, 1, 1, 2, 1, 1, 2, 2, 2, 2, 0, 1, 2, 0, 0, 1, 1, 2, 2, 1, 2, 1, 1, 1, 0, 2]
collector: received 2 encrypted aggregate share(s)
collector_decoded_slots=[0, 2, 3, 1, 3, 4, 0, 0, 2, 2, 2, 0, 4, 0, 4, 2, 4, 0, 4, 4, 0, 0, 2, 0, 2, 0, 2, 0, 4, 0, 2, 2, 2, 4, 2, 2, 4, 2, 2, 2, 2, 2, 4, 0, 2, 4, 4, 2, 2, 4, 2, 2, 4, 4, 4, 4, 0, 2, 4, 0, 0, 2, 2, 4, 4, 2, 4, 2, 2, 2, 0, 4]
collector_total=100 expected=100