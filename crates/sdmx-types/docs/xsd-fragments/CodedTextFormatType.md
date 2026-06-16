<details>
<summary>XSD contract: <code>CodedTextFormatType</code> (SDMX 3.0 and 3.1)</summary>

```xml
	<xs:complexType name="CodedTextFormatType">
		<xs:annotation>
			<xs:documentation>CodedTextFormatType is a restricted version of the SimpleComponentTextFormatType that only allows facets and text types applicable to codes. Although the time facets permit any value, an actual code identifier does not support the necessary characters for time. Therefore these facets should not contain time in their values.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="SimpleComponentTextFormatType">
				<xs:attribute name="textType" type="CodeDataType" use="optional"/>
				<xs:attribute name="isSequence" type="xs:boolean" use="optional"/>
				<xs:attribute name="interval" type="xs:integer" use="optional"/>
				<xs:attribute name="startValue" type="xs:integer" use="optional"/>
				<xs:attribute name="endValue" type="xs:integer" use="optional"/>
				<xs:attribute name="timeInterval" type="xs:duration" use="optional"/>
				<xs:attribute name="startTime" type="common:StandardTimePeriodType" use="optional"/>
				<xs:attribute name="endTime" type="common:StandardTimePeriodType" use="optional"/>
				<xs:attribute name="minLength" type="xs:positiveInteger" use="optional"/>
				<xs:attribute name="maxLength" type="xs:positiveInteger" use="optional"/>
				<xs:attribute name="minValue" type="xs:integer" use="optional"/>
				<xs:attribute name="maxValue" type="xs:integer" use="optional"/>
				<xs:attribute name="decimals" type="xs:positiveInteger" use="prohibited"/>
				<xs:attribute name="pattern" type="xs:string" use="optional"/>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
