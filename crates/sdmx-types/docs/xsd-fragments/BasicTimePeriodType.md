<details>
<summary>XSD contract: <code>BasicTimePeriodType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:simpleType name="BasicTimePeriodType">
		<xs:annotation>
			<xs:documentation>BasicTimePeriodType contains the basic dates and calendar periods. It is a combination of the Gregorian time periods and the date time type.</xs:documentation>
		</xs:annotation>
		<xs:union memberTypes="GregorianTimePeriodType xs:dateTime"/>
	</xs:simpleType>
```

</details>
