<details>
<summary>XSD contract: <code>TimeRangeValueType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="TimeRangeValueType">
		<xs:annotation>
			<xs:documentation>TimeRangeValueType allows a time period value to be expressed as a range. It can be expressed as the period before a period, after a period, or between two periods. Each of these properties can specify their inclusion in regards to the range.</xs:documentation>
		</xs:annotation>
		<xs:choice>
			<xs:element name="BeforePeriod" type="TimePeriodRangeType">
				<xs:annotation>
					<xs:documentation>BeforePeriod is the period before which the period is meant to cover. This date may be inclusive or exclusive in the range.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:element name="AfterPeriod" type="TimePeriodRangeType">
				<xs:annotation>
					<xs:documentation>AfterPeriod is the period after which the period is meant to cover. This date may be inclusive or exclusive in the range.</xs:documentation>
				</xs:annotation>
			</xs:element>
			<xs:sequence>
				<xs:element name="StartPeriod" type="TimePeriodRangeType">
					<xs:annotation>
						<xs:documentation>StartPeriod is the start date or the range that the queried date must occur within. This date may be inclusive or exclusive in the range.</xs:documentation>
					</xs:annotation>
				</xs:element>
				<xs:element name="EndPeriod" type="TimePeriodRangeType">
					<xs:annotation>
						<xs:documentation>EndPeriod is the end period of the range. This date may be inclusive or exclusive in the range.</xs:documentation>
					</xs:annotation>
				</xs:element>
			</xs:sequence>
		</xs:choice>
		<xs:attribute name="validFrom" type="common:StandardTimePeriodType" use="optional"/>
		<xs:attribute name="validTo" type="common:StandardTimePeriodType" use="optional"/>
	</xs:complexType>
```

</details>
