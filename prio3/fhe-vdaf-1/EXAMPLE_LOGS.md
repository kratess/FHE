client 0 input 0: value=51 encoded=[1, 1, 0, 0, 1, 1, 0] -> aggregator 1
2026-05-30T12:43:38.963165Z DEBUG fhe_vdaf_1: [ct] client_0_input_0 slots=7 decoded=[1, 1, 0, 0, 1, 1, 0]
client 0 input 1: value=7 encoded=[1, 1, 1, 0, 0, 0, 0] -> aggregator 1
2026-05-30T12:43:39.092496Z DEBUG fhe_vdaf_1: [ct] client_0_input_1 slots=7 decoded=[1, 1, 1, 0, 0, 0, 0]
client 0 input 2: value=4 encoded=[0, 0, 1, 0, 0, 0, 0] -> aggregator 1
2026-05-30T12:43:39.232947Z DEBUG fhe_vdaf_1: [ct] client_0_input_2 slots=7 decoded=[0, 0, 1, 0, 0, 0, 0]
client 1 input 0: value=49 encoded=[1, 0, 0, 0, 1, 1, 0] -> aggregator 0
2026-05-30T12:43:39.896027Z DEBUG fhe_vdaf_1: [ct] client_1_input_0 slots=7 decoded=[1, 0, 0, 0, 1, 1, 0]
client 1 input 1: value=13 encoded=[1, 0, 1, 1, 0, 0, 0] -> aggregator 1
2026-05-30T12:43:40.000905Z DEBUG fhe_vdaf_1: [ct] client_1_input_1 slots=7 decoded=[1, 0, 1, 1, 0, 0, 0]
aggregator 0: validating and aggregating 1 encrypted input(s)
2026-05-30T12:43:45.512639Z DEBUG fhe_vdaf_1: [ct] aggregator_0_aggregate_share slots=7 decoded=[1, 0, 0, 0, 1, 1, 0]
aggregator 1: validating and aggregating 4 encrypted input(s)
2026-05-30T12:44:16.036552Z DEBUG fhe_vdaf_1: [ct] aggregator_1_aggregate_share slots=7 decoded=[3, 2, 3, 1, 1, 1, 0]
collector: received 2 encrypted aggregate share(s)
collector_decoded_slots=[4, 2, 3, 1, 2, 2, 0]
collector_total=124 expected=124