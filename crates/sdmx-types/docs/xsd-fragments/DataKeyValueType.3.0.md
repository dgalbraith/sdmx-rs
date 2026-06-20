<details>
<summary>XSD contract: <code>DataKeyValueType</code> (SDMX 3.0)</summary>

```xml
	<xs:complexType name="DataKeyValueType">
		<xs:annotation>
			<xs:documentation>DataKeyValueType is a type for providing a dimension value for the purpose of defining a distinct data key. Only a single value can be provided for the dimension.</xs:documentation>
		</xs:annotation>
		<xs:complexContent>
			<xs:restriction base="MemberSelectionType">
				<xs:choice>
					<xs:element name="Value" type="SimpleKeyValueType"/>
				</xs:choice>
				<xs:attribute name="id" type="common:SingleNCNameIDType" use="required"/>
				<xs:attribute name="include" type="xs:boolean" use="optional" fixed="true"/>
				<xs:attribute name="validFrom" type="common:StandardTimePeriodType" use="prohibited"/>
				<xs:attribute name="validTo" type="common:StandardTimePeriodType" use="prohibited"/>
			</xs:restriction>
		</xs:complexContent>
	</xs:complexType>
```

</details>
