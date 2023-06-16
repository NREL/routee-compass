CREATE MATERIALIZED VIEW network_compass_v2 as
SELECT
    ntw_w_sp.feat_id,
    ntw_w_sp.junction_id_from,
    ntw_w_sp.junction_id_to,
    ntw_w_sp.centimeters,
    ntw_w_sp.routing_class,
    ntw_w_sp.mean_gradient_dec,
    sp.free_flow_speed,
    ntw_w_sp.validity_direction,
    ntw_w_sp.geom
FROM
(
    SELECT
        netw.feat_id,
        netw.junction_id_from,
        netw.junction_id_to,
        netw.centimeters,
        netw.routing_class,
        netw.mean_gradient_dec,
        nt2sp.speed_profile_id,
        nt2sp.validity_direction,
        netw.geom
    FROM
        (
            SELECT
                network.feat_id,
                network.junction_id_from,
                network.junction_id_to,
                network.centimeters,
                network.routing_class,
                network.mean_gradient_dec,
                network.geom
            FROM
                tomtom_multinet_current.network
        ) netw
        LEFT OUTER JOIN tomtom_multinet_current.mnr_netw2speed_profile nt2sp ON netw.feat_id = nt2sp.feat_id
) ntw_w_sp
LEFT OUTER JOIN tomtom_multinet_current.mnr_speed_profile sp ON ntw_w_sp.speed_profile_id = sp.speed_profile_id;