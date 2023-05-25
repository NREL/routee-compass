CREATE MATERIALIZED VIEW network_w_speed_profiles as
SELECT
    ntw_w_sp.netw_id,
    ntw_w_sp.junction_id_from,
    ntw_w_sp.junction_id_to,
    ntw_w_sp.centimeters,
    ntw_w_sp.routing_class,
    ntw_w_sp.mean_gradient_dec,
    ntw_w_sp.speed_average_pos,
    ntw_w_sp.speed_average_neg,
    sp.free_flow_speed,
    sp.monday_profile_id,
    sp.tuesday_profile_id,
    sp.wednesday_profile_id,
    sp.thursday_profile_id,
    sp.friday_profile_id,
    sp.saturday_profile_id,
    sp.sunday_profile_id,
    ntw_w_sp.link_direction,
    ntw_w_sp.geom
FROM
    (
        SELECT
            netw.netw_id,
            netw.junction_id_from,
            netw.junction_id_to,
            netw.centimeters,
            netw.routing_class,
            netw.mean_gradient_dec,
            netw.speed_average_pos,
            netw.speed_average_neg,
            nt2sp.speed_profile_id,
            nt2sp.validity_direction AS link_direction,
            netw.geom
        FROM
            (
                SELECT
                    network.feat_id AS netw_id,
                    network.junction_id_from,
                    network.junction_id_to,
                    network.centimeters,
                    network.routing_class,
                    network.mean_gradient_dec,
                    network.speed_average_pos,
                    network.speed_average_neg,
                    network.geom
                FROM
                    tomtom_multinet_current.network
                WHERE
                    network.routing_class <= 5 :: numeric
            ) netw
            LEFT OUTER JOIN tomtom_multinet_current.mnr_netw2speed_profile nt2sp ON netw.netw_id = nt2sp.netw_id
    ) ntw_w_sp
    LEFT OUTER JOIN tomtom_multinet_current.mnr_speed_profile sp ON ntw_w_sp.speed_profile_id = sp.speed_profile_id;