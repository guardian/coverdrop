#!/usr/bin/env python3

# This script is being used to compute estimages for the white
# paper (Appendix C). It relies on sensible input parameters.

u2j_senders = (500_000, 5_000_000)
u2j_msg_per_sender = (0.17, 0.17)  # msg/h
u2j_total_real_messages = (50, 100)  # msg/h
u2j_msg_size = 800  # bytes

u2j_cn_t_min = 100_000  # msgs
u2j_cn_t_max = 500_000  # msgs
u2j_cn_timeout = 1  # h
u2j_cn_output_batch_size = 500  # msgs

j2u_senders = (5, 40)
j2u_msg_per_sender = (1, 2)
j2u_total_real_messages = (5, 15)
j2u_msg_size = 600

j2u_cn_t_min = 20
j2u_cn_t_max = 50
j2u_cn_timeout = 1
j2u_cn_output_batch_size = 20


def calc(
    senders,
    msg_per_sender,
    total_real_messages,
    msg_size,
    cn_t_min,
    cn_t_max,
    cn_timeout,
    cn_output_batch_size
):
    print("[General estimates]")
    print(f"Senders: {senders[0]:,} - {senders[1]:,}")
    print(f"Messages per sender: {msg_per_sender[0]:.2f} - {msg_per_sender[1]:.2f} msg/h")
    print(f"Total real messages: {total_real_messages[0]:,} - {total_real_messages[1]:,} msg/h")
    print(f"Message size: {msg_size} bytes")

    print()
    print("[Sending messages (over all users)]")

    expected_messages_per_hour = (
        senders[0] * msg_per_sender[0],
        senders[1] * msg_per_sender[1],
    )
    print(f"Expected messages per hour: {expected_messages_per_hour[0]:,.0f} - {expected_messages_per_hour[1]:,.0f}")

    ingress_at_cdn_mib = (
        expected_messages_per_hour[0] * msg_size / 1024 / 1024,
        expected_messages_per_hour[1] * msg_size / 1024 / 1024,
    )
    print(f"Ingress at CDN: {ingress_at_cdn_mib[0]:,.2f} - {ingress_at_cdn_mib[1]:,.2f} MiB/h")

    ingress_at_cdn_kib_per_second = (
        ingress_at_cdn_mib[0] / 3600 * 1024,
        ingress_at_cdn_mib[1] / 3600 * 1024,
    )
    print(f"Ingress at CDN: {ingress_at_cdn_kib_per_second[0]:,.2f} - {ingress_at_cdn_kib_per_second[1]:,.2f} KiB/s")

    ratio_real_messages = sorted(
        (
            total_real_messages[0] / expected_messages_per_hour[0],
            total_real_messages[1] / expected_messages_per_hour[1],
        )
    )
    print(f"Ratio real messages: {100*ratio_real_messages[0]:.2f}% - {100*ratio_real_messages[1]:.2f}%")

    print()
    print("[CoverNode parameters]")
    print(f"t_min: {cn_t_min:,} messages")
    print(f"t_max: {cn_t_max:,} messages")
    print(f"Timeout: {cn_timeout} h")
    print(f"Output batch size: {cn_output_batch_size} messages")

    print()
    print("[CoverNode processing]")

    firing_rates = []
    firing_rates.append(max((max(expected_messages_per_hour) / cn_t_max), cn_timeout))
    firing_rates.append(max((min(expected_messages_per_hour) / cn_t_min), cn_timeout))
    print(f"Firing rate: {min(firing_rates):.2f} - {max(firing_rates):.2f} 1/h")

    message_delays = (
        1 / max(firing_rates) / 2,
        1 / min(firing_rates) / 2,
    )
    print(f"Mean message delay: {message_delays[0]:.2f} - {message_delays[1]:.2f} h")

    output_rates = (
        cn_output_batch_size * min(firing_rates),
        cn_output_batch_size * max(firing_rates),
    )
    print(f"Output rate: {output_rates[0]:.0f} - {output_rates[1]:.0f} messages/h")

    ratio_real_messages_out = sorted(
        (
            total_real_messages[0] / output_rates[0],
            total_real_messages[1] / output_rates[1],
        )
    )
    print(f"Ratio real messages: {100*ratio_real_messages_out[0]:.0f}% - {100*ratio_real_messages_out[1]:.0f}%")

    print()
    print("[Received messages (for opposite side)]")

    published_batches = (
        min(firing_rates),
        max(firing_rates),
    )
    print(f"Published batches: {published_batches[0]:.0f} - {published_batches[1]:.0f} batches/h")

    size_of_batch_mib = cn_output_batch_size * msg_size / 1024 / 1024
    print(f"Size of batch: {size_of_batch_mib:.2f} MiB")

    total_egress_for_receiving_side = (
        published_batches[0] * size_of_batch_mib,
        published_batches[1] * size_of_batch_mib,
    )
    print(f"Total egress for receiving side: {total_egress_for_receiving_side[0]:,.2f} - {total_egress_for_receiving_side[1]:,.2f} MiB/h")

    total_egress_per_month = (
        total_egress_for_receiving_side[0] * 24 * 30,
        total_egress_for_receiving_side[1] * 24 * 30,
    )
    print(f"Total egress per month: {total_egress_per_month[0]:,.2f} - {total_egress_per_month[1]:,.2f} MiB/month")


def run():
    print("-- U2J --")
    calc(
        senders=u2j_senders,
        msg_per_sender=u2j_msg_per_sender,
        total_real_messages=u2j_total_real_messages,
        msg_size=u2j_msg_size,
        cn_t_min=u2j_cn_t_min,
        cn_t_max=u2j_cn_t_max,
        cn_timeout=u2j_cn_timeout,
        cn_output_batch_size=u2j_cn_output_batch_size
    )

    print()
    print("-- J2U --")
    calc(
        senders=j2u_senders,
        msg_per_sender=j2u_msg_per_sender,
        total_real_messages=j2u_total_real_messages,
        msg_size=j2u_msg_size,
        cn_t_min=j2u_cn_t_min,
        cn_t_max=j2u_cn_t_max,
        cn_timeout=j2u_cn_timeout,
        cn_output_batch_size=j2u_cn_output_batch_size
    )


if __name__ == '__main__':
    run()
