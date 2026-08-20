# GPS

GPS readme placeholder.

## UBX

### UBX-NAV

Navigation Results Messages: i.e. Position, Speed, Time, Acceleration, Heading, DOP, SVs used.
Messages in the NAV class are used to output navigation data such as position, altitude and
velocity in a number of formats.
Additionally, status flags and accuracy figures are output.
The messages are generated with the configured navigation/measurement rate.

### UBX-NAV-SOL

This message has only been retained for backwards compatibility; users are recommended to use the UBX-NAV-PVT message in preference.

### UBX-NAV-SVINFO

This message has only been retained for backwards compatibility; users are recommended to use the UBX-NAV-SAT message in preference.
