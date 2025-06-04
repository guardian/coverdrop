# AWS Instance Reservations

It's that time of the year again.

Instance reservation is performed around February/March every year to reserve instance for the next 12 months. Reserving an instance during the reservation cycle is always cheaper than provisioning one on-demand (you can compare reserved and on-demand costs [here](https://instances.vantage.sh)).

Tests in [`reservations.test.ts`](../cdk/lib/reservations.test.ts) ensure that the Infrastructure as Code stays up to date with the instances that have been reserved manually in the AWS console.


When reserving instances, follow these steps:

1. Go to [`reservations.ts`](../cdk/lib/reservations.ts)
2. Determine your appropriate instance classes and sizes for the following year. If there are any `nonReserved` instances from the previous year, determine whether it's appropriate to cull them or to reserve them for the following year. Should you choose to reserve them (maybe because you forgot to do it the previous year) simply remove them from the `nonReserved` array and make sure they are included in the relevant EC2 or RDS field within the `reservations` object.
3. Do the reservations in the AWS console (guidance for this is usually provided by the DevX team). 
4. Update the `RESERVATIONS_VALID_UNTIL_DATE` const in [`reservations.ts`](../cdk/lib/reservations.ts) with the new date.
5. Update the values in the `reservations` object in [`reservations.ts`](../cdk/lib/reservations.ts) with what you have reserved in the console. 

### What happens if I forget to do this?
Your tests will simply start failing after the `RESERVATIONS_VALID_UNTIL_DATE` has passed, effectively blocking your pipeline.

This friction is by design - it only happens once a year, so it is designed to be a minor nuisance. A warning will start appearing in the console one month prior to the date to give you enough time to perform the reservations.
This process ensures that you spend the appropriate amount of time to make an informed decisions about the instances required for the following year, and no money  (or as little as possible) is wasted on on-demand instances.
