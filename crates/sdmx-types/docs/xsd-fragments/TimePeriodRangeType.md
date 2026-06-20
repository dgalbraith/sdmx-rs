<details>
<summary>XSD contract: <code>TimePeriodRangeType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="TimePeriodRangeType">
		<xs:annotation>
			<xs:documentation>TimePeriodRangeType defines a time period, and indicates whether it is inclusive in a range.</xs:documentation>
		</xs:annotation>
		<xs:simpleContent>
			<xs:extension base="common:ObservationalTimePeriodType">
				<xs:attribute name="isInclusive" type="xs:boolean" default="true">
					<xs:annotation>
						<xs:documentation>The isInclusive attribute, when true, indicates that the time period specified is included in the range.</xs:documentation>
					</xs:annotation>
				</xs:attribute>
			</xs:extension>
		</xs:simpleContent>
	</xs:complexType>
```

</details>
