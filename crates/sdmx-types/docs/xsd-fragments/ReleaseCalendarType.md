<details>
<summary>XSD contract: <code>ReleaseCalendarType</code> (SDMX 3.0)</summary>

```xml
	<xs:complexType name="ReleaseCalendarType">
		<xs:annotation>
			<xs:documentation>ReleaseCalendarType describes information about the timing of releases of the constrained data. All of these values use the standard "P7D" - style format.</xs:documentation>
		</xs:annotation>
		<xs:sequence>
			<xs:element name="Periodicity" type="xs:string">
				<xs:annotation>
					<xs:documentation>Periodicity is the period between releases of the data set.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:element name="Offset" type="xs:string">
				<xs:annotation>
					<xs:documentation>Offset is the interval between January first and the first release of data within the year.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:element name="Tolerance" type="xs:string">
				<xs:annotation>
					<xs:documentation>Tolerance is the period after which the release of data may be deemed late.</xs:documentation>
				</xs:annotation>
			</xs:element>
		</xs:sequence>
	</xs:complexType>
```

</details>
